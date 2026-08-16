use std::collections::HashMap;
use std::f32::consts::PI;
use std::path::Path;

use anyhow::{Context, Result, bail};
use mlx_rs::fast;
use mlx_rs::ops;
use mlx_rs::ops::indexing::IndexOp;
use mlx_rs::{Array, Dtype, Stream, StreamOrDevice};

use super::InferenceBackend;
use super::config::ModelConfig;

const SWIGLU_ALPHA: f32 = 1.702;
const SWIGLU_LIMIT: f32 = 7.0;

fn yarn_freqs(head_dim: usize, cfg: &ModelConfig) -> Vec<f32> {
    let dims = head_dim as f64;
    let half = head_dim / 2;
    let base = cfg.rope_parameters.rope_theta;
    let factor = cfg.rope_parameters.factor;
    let beta_fast = cfg.rope_parameters.beta_fast;
    let beta_slow = cfg.rope_parameters.beta_slow;
    let orig = cfg.rope_parameters.original_max_position_embeddings as f64;

    let corr_dim = |n: f64| dims * (orig / (n * 2.0 * PI as f64)).ln() / (2.0 * base.ln());
    let low = corr_dim(beta_fast).floor().max(0.0).min(dims - 1.0);
    let high = corr_dim(beta_slow).ceil().max(low + 1.0).min(dims - 1.0);

    let mut freqs = Vec::with_capacity(half);
    for i in 0..half {
        let freq_extra = base.powf(2.0 * i as f64 / dims);
        let freq_inter = factor * freq_extra;
        let ramp = ((i as f64 - low) / (high - low)).clamp(0.0, 1.0);
        let mask = 1.0 - ramp;
        let f = (freq_inter * freq_extra) / (freq_inter * mask + freq_extra * (1.0 - mask));
        freqs.push(f as f32);
    }
    freqs
}

fn yarn_mscale(cfg: &ModelConfig) -> f32 {
    let factor = cfg.rope_parameters.factor;
    if factor > 1.0 {
        (0.1 * factor.ln() + 1.0) as f32
    } else {
        1.0
    }
}

struct Linear {
    weight_t: Array,
    bias: Array,
}

impl Linear {
    fn load(weights: &HashMap<String, Array>, prefix: &str) -> Result<Self> {
        let weight = weights
            .get(&format!("{prefix}.weight"))
            .with_context(|| format!("missing weight {prefix}.weight"))?;
        let bias = weights
            .get(&format!("{prefix}.bias"))
            .with_context(|| format!("missing weight {prefix}.bias"))?;
        Ok(Self {
            weight_t: weight.swap_axes_device(-1, -2, StreamOrDevice::default())?,
            bias: bias.clone(),
        })
    }

    fn forward(&self, x: &Array, stream: &Stream) -> Result<Array> {
        let y = x.matmul_device(&self.weight_t, stream)?;
        Ok(y.add_device(&self.bias, stream)?)
    }
}

struct SwitchLinear {
    weight: Array,
    bias: Array,
}

impl SwitchLinear {
    fn load(weights: &HashMap<String, Array>, prefix: &str) -> Result<Self> {
        let weight = weights
            .get(&format!("{prefix}.weight"))
            .with_context(|| format!("missing weight {prefix}.weight"))?;
        let bias = weights
            .get(&format!("{prefix}.bias"))
            .with_context(|| format!("missing weight {prefix}.bias"))?;
        Ok(Self {
            weight: weight.clone(),
            bias: bias.clone(),
        })
    }

    fn forward(&self, x: &Array, indices: &Array, sorted: bool, stream: &Stream) -> Result<Array> {
        let weight_t = self.weight.swap_axes_device(-1, -2, stream)?;
        let out = gather_mm(x, &weight_t, indices, sorted, stream)?;
        let bias = self.bias.take_axis_device(indices, 0, stream)?;
        let bias = bias.expand_dims_device(-2, stream)?;
        Ok(out.add_device(&bias, stream)?)
    }
}

fn gather_mm(
    a: &Array,
    b: &Array,
    rhs_indices: &Array,
    sorted: bool,
    stream: &Stream,
) -> Result<Array> {
    let mut res = unsafe { mlx_sys::mlx_array_new() };
    let status = unsafe {
        mlx_sys::mlx_gather_mm(
            &mut res,
            a.as_ptr(),
            b.as_ptr(),
            mlx_sys::mlx_array_new(),
            rhs_indices.as_ptr(),
            sorted,
            stream.as_ptr(),
        )
    };
    if status != 0 {
        bail!("mlx_gather_mm failed with status {status}");
    }
    Ok(unsafe { Array::from_ptr(res) })
}

fn sdpa(
    q: &Array,
    k: &Array,
    v: &Array,
    scale: f32,
    mask: &Array,
    sinks: &Array,
    stream: &Stream,
) -> Result<Array> {
    let mut res = unsafe { mlx_sys::mlx_array_new() };
    let status = unsafe {
        mlx_sys::mlx_fast_scaled_dot_product_attention(
            &mut res,
            q.as_ptr(),
            k.as_ptr(),
            v.as_ptr(),
            scale,
            c"".as_ptr(),
            mask.as_ptr(),
            sinks.as_ptr(),
            stream.as_ptr(),
        )
    };
    if status != 0 {
        bail!("mlx_fast_scaled_dot_product_attention failed with status {status}");
    }
    Ok(unsafe { Array::from_ptr(res) })
}

struct Attention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    sinks: Array,
    freqs: Array,
    mscale: f32,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
}

impl Attention {
    fn forward(&self, x: &Array, mask: &Array, stream: &Stream) -> Result<Array> {
        let n = x.shape()[1];
        let heads = self.num_heads as i32;
        let d = self.head_dim as i32;

        let q = self
            .q_proj
            .forward(x, stream)?
            .reshape_device(&[1, n, heads, d], stream)?;
        let k = self
            .k_proj
            .forward(x, stream)?
            .reshape_device(&[1, n, self.num_kv_heads as i32, d], stream)?;
        let v = self
            .v_proj
            .forward(x, stream)?
            .reshape_device(&[1, n, self.num_kv_heads as i32, d], stream)?;

        let q = q.swap_axes_device(1, 2, stream)?;
        let k = k.swap_axes_device(1, 2, stream)?;
        let v = v.swap_axes_device(1, 2, stream)?;

        let q = fast::rope_device(
            &q,
            d,
            true,
            None::<f32>,
            self.mscale,
            0,
            Some(&self.freqs),
            stream,
        )?;
        let k = fast::rope_device(
            &k,
            d,
            true,
            None::<f32>,
            self.mscale,
            0,
            Some(&self.freqs),
            stream,
        )?;

        let scale = 1.0 / (self.head_dim as f32).sqrt();
        let sinks = self.sinks.as_dtype_device(q.dtype(), stream)?;
        let out = sdpa(&q, &k, &v, scale, mask, &sinks, stream)?;
        let out = out
            .swap_axes_device(1, 2, stream)?
            .reshape_device(&[1, n, heads * d], stream)?;
        self.o_proj.forward(&out, stream)
    }
}

struct Experts {
    gate_proj: SwitchLinear,
    up_proj: SwitchLinear,
    down_proj: SwitchLinear,
    num_experts_per_tok: usize,
}

impl Experts {
    fn forward(&self, x: &Array, indices: &Array, stream: &Stream) -> Result<Array> {
        let idx_shape = indices.shape().to_vec();
        let (b, n, k) = (idx_shape[0], idx_shape[1], idx_shape[2]);
        let hidden = x.shape()[2];

        let mut x = x.reshape_device(&[b, n, 1, 1, hidden], stream)?;
        let total = (n * k) as usize;

        let do_sort = total >= 64;
        let mut inv_order: Option<Array> = None;

        let idx = if do_sort {
            let flat_idx = indices.reshape_device(&[-1], stream)?;
            let order = ops::argsort_device(&flat_idx, stream)?;
            let row = order.floor_divide_device(&Array::from(k), stream)?;
            let flat = x.reshape_device(&[b * n, 1, hidden], stream)?;
            x = flat.take_axis_device(&row, 0, stream)?;
            inv_order = Some(ops::argsort_device(&order, stream)?);
            flat_idx.take_axis_device(&order, 0, stream)?
        } else {
            indices.clone()
        };

        let x_up = self.up_proj.forward(&x, &idx, do_sort, stream)?;
        let x_gate = self.gate_proj.forward(&x, &idx, do_sort, stream)?;

        let alpha = Array::from_f32(SWIGLU_ALPHA).as_dtype_device(x.dtype(), stream)?;
        let one = Array::from_f32(1.0).as_dtype_device(x.dtype(), stream)?;
        let limit = Array::from_f32(SWIGLU_LIMIT).as_dtype_device(x.dtype(), stream)?;
        let neg_limit = Array::from_f32(-SWIGLU_LIMIT).as_dtype_device(x.dtype(), stream)?;
        let gate = ops::clip_device(&x_gate, ((), &limit), stream)?;
        let up = ops::clip_device(&x_up, (&neg_limit, &limit), stream)?;
        let glu = gate.multiply_device(
            &ops::sigmoid_device(&gate.multiply_device(&alpha, stream)?, stream)?,
            stream,
        )?;
        let act = up.add_device(&one, stream)?.multiply_device(&glu, stream)?;

        let mut out = self.down_proj.forward(&act, &idx, do_sort, stream)?;

        if let Some(inv) = inv_order {
            let unsorted = out.take_axis_device(&inv, 0, stream)?;
            out = unsorted.reshape_device(&[b, n, k, 1, hidden], stream)?;
        } else {
            out = out.reshape_device(&[b, n, k, 1, hidden], stream)?;
        }

        out.squeeze_axes_device(&[-2], stream).map_err(Into::into)
    }
}

struct EncoderLayer {
    attn: Attention,
    gate_up_down: Experts,
    router: Linear,
    input_norm: Array,
    post_norm: Array,
    eps: f32,
}

impl EncoderLayer {
    fn rms_norm(&self, x: &Array, weight: &Array, stream: &Stream) -> Result<Array> {
        fast::rms_norm_device(x, weight, self.eps, stream).map_err(Into::into)
    }

    fn forward(&self, x: &Array, mask: &Array, stream: &Stream) -> Result<Array> {
        let h = self.rms_norm(x, &self.input_norm, stream)?;
        let h = self.attn.forward(&h, mask, stream)?;
        let x = x.add_device(&h, stream)?;

        let h = self.rms_norm(&x, &self.post_norm, stream)?;
        let h = self.mlp_forward(&h, stream)?;
        x.add_device(&h, stream).map_err(Into::into)
    }

    fn mlp_forward(&self, x: &Array, stream: &Stream) -> Result<Array> {
        let router_logits = self
            .router
            .forward(x, stream)?
            .as_dtype_device(Dtype::Float32, stream)?;

        let num_experts = router_logits.shape()[2];
        let k = self.gate_up_down.num_experts_per_tok as i32;
        let top_idx = ops::argpartition_axis_device(&router_logits, -k, -1, stream)?.index((
            ..,
            ..,
            (num_experts - k)..num_experts,
        ));
        let top_val = ops::indexing::take_along_axis_device(&router_logits, &top_idx, -1, stream)?;
        let weights = ops::softmax_axis_device(&top_val, -1, false, stream)?
            .as_dtype_device(x.dtype(), stream)?;
        let y = self.gate_up_down.forward(x, &top_idx, stream)?;
        let w = weights.expand_dims_device(-1, stream)?;
        let y = y.multiply_device(&w, stream)?;
        ops::sum_axis_device(&y, -2, false, stream).map_err(Into::into)
    }
}

pub struct MlxBackend {
    embed: Array,
    layers: Vec<EncoderLayer>,
    norm: Array,
    score: Linear,
    eps: f32,
    sliding_window: usize,
    num_labels: usize,
    stream: Stream,
}

unsafe impl Send for MlxBackend {}

impl MlxBackend {
    pub fn load(model_dir: &Path, config: &ModelConfig) -> Result<Self> {
        let stream = StreamOrDevice::default().as_ref().clone();
        let weights = load_weights(model_dir)?;

        let embed = take(&weights, "model.embed_tokens.weight")?.clone();
        let norm = take(&weights, "model.norm.weight")?.clone();
        let score = Linear::load(&weights, "score")?;

        let head_dim = if config.head_dim > 0 {
            config.head_dim
        } else {
            config.hidden_size / config.num_attention_heads
        };
        let freqs = Array::from_slice(&yarn_freqs(head_dim, config), &[(head_dim / 2) as i32]);
        let mscale = yarn_mscale(config);

        let mut layers = Vec::with_capacity(config.num_hidden_layers);
        for i in 0..config.num_hidden_layers {
            let prefix = format!("model.layers.{i}");
            let attn = Attention {
                q_proj: Linear::load(&weights, &format!("{prefix}.self_attn.q_proj"))?,
                k_proj: Linear::load(&weights, &format!("{prefix}.self_attn.k_proj"))?,
                v_proj: Linear::load(&weights, &format!("{prefix}.self_attn.v_proj"))?,
                o_proj: Linear::load(&weights, &format!("{prefix}.self_attn.o_proj"))?,
                sinks: take(&weights, &format!("{prefix}.self_attn.sinks"))?.clone(),
                freqs: freqs.clone(),
                mscale,
                num_heads: config.num_attention_heads,
                num_kv_heads: config.num_key_value_heads,
                head_dim,
            };
            let experts = Experts {
                gate_proj: SwitchLinear::load(
                    &weights,
                    &format!("{prefix}.mlp.experts.gate_proj"),
                )?,
                up_proj: SwitchLinear::load(&weights, &format!("{prefix}.mlp.experts.up_proj"))?,
                down_proj: SwitchLinear::load(
                    &weights,
                    &format!("{prefix}.mlp.experts.down_proj"),
                )?,
                num_experts_per_tok: config.num_experts_per_tok,
            };
            layers.push(EncoderLayer {
                attn,
                gate_up_down: experts,
                router: Linear::load(&weights, &format!("{prefix}.mlp.router"))?,
                input_norm: take(&weights, &format!("{prefix}.input_layernorm.weight"))?.clone(),
                post_norm: take(
                    &weights,
                    &format!("{prefix}.post_attention_layernorm.weight"),
                )?
                .clone(),
                eps: config.rms_norm_eps as f32,
            });
        }

        Ok(Self {
            embed,
            layers,
            norm,
            score,
            eps: config.rms_norm_eps as f32,
            sliding_window: config.sliding_window,
            num_labels: config.num_labels(),
            stream,
        })
    }

    fn build_mask(&self, n: i32, dtype: Dtype) -> Result<Array> {
        let s = &self.stream;
        let idx = ops::arange_device::<i32, i32>(0, n, 1, s)?;
        let rows = idx.reshape_device(&[n, 1], s)?;
        let cols = idx.reshape_device(&[1, n], s)?;
        let diff = rows.subtract_device(&cols, s)?.abs_device(s)?;
        let local = diff.le_device(&Array::from(self.sliding_window as i32), s)?;
        let mask = ops::r#where_device(
            &local,
            &Array::from_f32(0.0),
            &Array::from_f32(f32::NEG_INFINITY),
            s,
        )?;
        let mask = mask.as_dtype_device(dtype, s)?;
        mask.reshape_device(&[1, 1, n, n], s).map_err(Into::into)
    }
}

impl InferenceBackend for MlxBackend {
    fn forward_logits(&mut self, ids: &[u32]) -> Result<Vec<Vec<f32>>> {
        let s = &self.stream;
        let n = ids.len() as i32;
        let ids_i32: Vec<i32> = ids.iter().map(|&i| i as i32).collect();
        let ids_arr = Array::from_slice(&ids_i32, &[n]);

        let mut h = self.embed.take_axis_device(&ids_arr, 0, s)?;
        h = h.reshape_device(&[1, n, self.embed.shape()[1]], s)?;

        let mask = self.build_mask(n, h.dtype())?;

        for layer in &self.layers {
            h = layer.forward(&h, &mask, s)?;
        }

        let h = fast::rms_norm_device(&h, &self.norm, self.eps, s)?;
        let logits = self.score.forward(&h, s)?;
        let logits = logits
            .reshape_device(&[n, self.num_labels as i32], s)?
            .as_dtype_device(Dtype::Float32, s)?;
        logits.eval()?;

        let flat = logits.as_slice::<f32>();
        let width = self.num_labels;
        Ok(flat.chunks(width).map(|c| c.to_vec()).collect())
    }
}

fn take<'a>(weights: &'a HashMap<String, Array>, key: &str) -> Result<&'a Array> {
    weights
        .get(key)
        .with_context(|| format!("missing weight {key}"))
}

fn load_weights(model_dir: &Path) -> Result<HashMap<String, Array>> {
    let mut files: Vec<_> = std::fs::read_dir(model_dir)
        .with_context(|| format!("failed to read {}", model_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "safetensors").unwrap_or(false))
        .collect();
    files.sort();
    if files.is_empty() {
        bail!("no .safetensors files in {}", model_dir.display());
    }

    let mut weights = HashMap::new();
    for file in files {
        let map: HashMap<String, Array> = Array::load_safetensors(&file)
            .map_err(|e| anyhow::anyhow!("failed to load {}: {e}", file.display()))?;
        for (k, v) in map {
            if !k.starts_with("original.") {
                weights.insert(k, v);
            }
        }
    }
    Ok(weights)
}
