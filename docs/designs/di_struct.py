from __future__ import annotations

import re
from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field, field_validator


class ActionCode(str, Enum):
    D = "D"
    Z = "Z"
    X = "X"
    K = "K"
    C = "C"
    U = "U"
    Z_D = "Z/D"
    X_Z = "X/Z"
    X_D = "X/D"
    X_Z_D = "X/Z/D"
    X_Z_U_STAR = "X/Z/U*"


class ProfileOptions(BaseModel):
    retain_safe_private: Optional[ActionCode] = None
    retain_uids: Optional[ActionCode] = None
    retain_dev_id: Optional[ActionCode] = None
    retain_inst_id: Optional[ActionCode] = None
    retain_pat_chars: Optional[ActionCode] = None
    retain_long_full_dates: Optional[ActionCode] = None
    retain_long_mod_dates: Optional[ActionCode] = None
    clean_desc: Optional[ActionCode] = None
    clean_struct_cont: Optional[ActionCode] = None
    clean_graph: Optional[ActionCode] = None


class TableE11Rule(BaseModel):
    tag: str = Field(..., description="DICOM tag in formatted string, e.g. (0010,0010)")
    attribute_name: str
    retired: bool = False
    in_std_comp_iod: bool = True
    basic_profile: ActionCode
    options: ProfileOptions = Field(default_factory=ProfileOptions)

    @field_validator("tag")
    @classmethod
    def normalize_tag(cls, v: str) -> str:
        match = re.match(r"^\(?([0-9A-Fa-f]{4})[, ]?([0-9A-Fa-f]{4})\)?$", v.strip())
        if not match:
            raise ValueError(f"Invalid DICOM tag format: {v}")
        return f"({match.group(1).upper()},{match.group(2).upper()})"

    @property
    def int_tuple(self) -> tuple[int, int]:
        clean = self.tag.strip("()")
        g, e = clean.split(",")
        return int(g, 16), int(e, 16)


class DeidentificationConfig(BaseModel):
    """Runtime flags determining which PS3.15 Annex E options are enabled."""
    retain_safe_private: bool = False
    retain_uids: bool = False
    retain_dev_id: bool = False
    retain_inst_id: bool = False
    retain_pat_chars: bool = False
    retain_long_full_dates: bool = False
    retain_long_mod_dates: bool = False
    clean_desc: bool = False
    clean_struct_cont: bool = False
    clean_graph: bool = False
    