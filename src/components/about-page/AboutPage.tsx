import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getVersion } from "@tauri-apps/api/app";
import KarmaLogo from "../icons/KarmaLogo";

export const AboutPage: React.FC = () => {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion("0.1.0"));
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto flex flex-col items-center gap-8 pt-6">
      <div className="flex flex-col items-center gap-3">
        <KarmaLogo width={72} />
        <h1 className="text-2xl font-semibold tracking-tight">Karma</h1>
        <p className="text-sm text-mid-gray">{t("about.tagline")}</p>
        <span className="text-xs font-mono text-mid-gray/70 bg-mid-gray/10 px-2 py-0.5 rounded">
          v{version}
        </span>
      </div>

      <div className="w-full max-w-md">
        <h2 className="text-xs font-medium uppercase tracking-wider text-mid-gray mb-3">
          {t("about.poweredBy")}
        </h2>
        <div className="grid grid-cols-2 gap-2">
          {[
            { name: "MLX", desc: t("about.tech.mlx") },
            { name: "Tauri", desc: t("about.tech.tauri") },
            { name: "MLX Embeddings", desc: t("about.tech.mlxEmbeddings") },
            { name: "React", desc: t("about.tech.react") },
          ].map((item) => (
            <div
              key={item.name}
              className="flex flex-col gap-0.5 p-2.5 rounded-lg bg-mid-gray/8 border border-mid-gray/12"
            >
              <span className="text-sm font-medium">{item.name}</span>
              <span className="text-xs text-mid-gray">{item.desc}</span>
            </div>
          ))}
        </div>
      </div>

      <p className="text-xs text-mid-gray/50 pb-4">
        {t("about.credit")}
      </p>
    </div>
  );
};
