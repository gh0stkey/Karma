"""Entry point: python -m opf_onnx"""

from opf_onnx.redactor import PIIRedactor
from opf_common.ipc import main

main(PIIRedactor)
