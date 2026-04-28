"""Entry point: python -m opf_mlx"""

from opf_mlx.redactor import PIIRedactor
from opf_common.ipc import main

main(PIIRedactor)
