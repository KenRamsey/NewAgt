"""Smoke tests for the ``newagt`` PyO3 module."""

from __future__ import annotations

import unittest
from pathlib import Path

import newagt

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "crates/newagt-core/tests/fixtures/prototype_tgt_sect_snippet.agt"
TGT_DAT = ROOT / "crates/newagt-core/tests/fixtures/tgt.dat.snippet"


class TestNewagtImport(unittest.TestCase):
    def test_version(self) -> None:
        self.assertTrue(newagt.__version__)

    def test_parse_fixture_summary(self) -> None:
        doc = newagt.parse(str(FIXTURE))
        summary = doc.summary()
        self.assertEqual(summary["sections"]["tgt_sect"], 1)
        self.assertGreaterEqual(summary["targets"], 1)

    def test_validate_loadable(self) -> None:
        report = newagt.validate(str(FIXTURE))
        self.assertTrue(report["loadable"])
        self.assertIsInstance(report["entries"], list)

    def test_bbox_count_pixbox(self) -> None:
        records = newagt.bboxes(
            str(FIXTURE),
            640,
            480,
            30.0,
            20.0,
            tgt_dat=str(TGT_DAT),
            method="score",
        )
        self.assertGreaterEqual(len(records), 1)
        bbox = records[0]["bbox"]
        self.assertEqual(bbox["provenance"], "pix_box")


if __name__ == "__main__":
    unittest.main()
