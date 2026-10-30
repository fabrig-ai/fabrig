"""Canonical coordinator routes (tools/queue/design.md, 3.11)."""

import importlib.util
import os
from pathlib import Path
import unittest

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("route_publication", DIRECTORY / "publication.py")
pub = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pub)
ROSTER = b"coord s/owner@fixture.example\nbeta -\n"


class RouteTests(unittest.TestCase):
    def test_route_resolved(self):
        self.assertEqual(pub.route(pub.coordinators(ROSTER), "ready/coord/docs"),
                         {"route_status": "resolved", "coordinator": "s/owner@fixture.example"})

    def test_route_unassigned(self):
        self.assertEqual(pub.route(pub.coordinators(ROSTER), "ready/beta/docs"),
                         {"route_status": "unresolved", "route_reason": "unassigned"})

    def test_route_missing_key(self):
        self.assertEqual(pub.route(pub.coordinators(ROSTER), "ready/absent/docs"),
                         {"route_status": "unresolved", "route_reason": "missing_key"})

    def test_route_historical_unknown(self):
        self.assertIsNone(pub.validate_route({"route_status": "unresolved", "route_reason": "historical_unknown"}))
        self.assertEqual(pub.route({}, "ready/coord/docs")["route_reason"], "missing_key")

    def test_route_invalid_wire(self):
        rows = [None, [], {}, {"coordinator": "x@fixture.example"},
                {"route_status": "unresolved"}, {"route_status": "resolved"},
                {"route_status": "unresolved", "route_reason": "other"},
                {"route_status": "unresolved", "route_reason": None},
                {"route_status": "unresolved", "route_reason": "unassigned", "coordinator": "x@fixture.example"},
                {"route_status": "resolved", "coordinator": "x@fixture.example", "route_reason": "unassigned"}]
        rows += [{"route_status": "resolved", "coordinator": value}
                 for value in (None, "", "-", "unknown", "local", 1, {}, [])]
        for value in rows:
            with self.subTest(value=value), self.assertRaises(ValueError):
                pub.validate_route(value)

    def test_strict_coordinator_config(self):
        rows = [b"coord local\n", b"coord x@host extra\n", b"coord x@host\ncoord -\n",
                b"bad/key x@host\n", b"coord x@host\r\n", b"coord x@host\x00\n", b"# hidden\xc2\x85control\n", b"\xff"]
        for value in rows:
            with self.subTest(value=value), self.assertRaises(ValueError):
                pub.coordinators(value)
        self.assertEqual(pub.coordinators(b"# routes\n\ncoord\tx@host # saved\n"), {"coord": "x@host"})

    def test_credential_free_origins(self):
        self.assertEqual(pub.explicit_origin("/fixture/origin with space.git"), "/fixture/origin with space.git")
        for value in ("https://user:password@fixture.example/repo", "https://token@fixture.example/repo",
                      "ext::helper", "-origin", "ssh://user:password@host/repo"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                pub.explicit_origin(value)

    def test_reviewed_roster_preserves_existing_routes_and_two_column_leads(self):
        leads = {}
        for row in (DIRECTORY / "leads.txt").read_text().splitlines():
            cells = row.split("#", 1)[0].split()
            if cells:
                self.assertEqual(len(cells), 2)
                key, address = cells
                self.assertNotIn(key, leads)
                self.assertRegex(key, pub.KEY)
                self.assertRegex(address, pub.authority.ADDRESS)
                leads[key] = address
        expected_leads = {
            "coord": "s/dynamic-memory@hel8", "ops": "ops@hel8",
            "mc": "s/multi-coord@hel8", "q": "s/qa@hel8", "u": "s/upgrades@hel8",
        }
        expected_coordinators = dict(expected_leads)
        for tree in ("skein", "smith", "jig", "temper"):
            expected_leads[tree] = f"lead/{tree}@hel8"
            expected_coordinators[tree] = "s/dynamic-memory@hel8"
            for group, owner in (("q", "s/qa@hel8"), ("u", "s/upgrades@hel8")):
                expected_leads[f"{group}-{tree}"] = f"lead/{group}/{tree}@hel8"
                expected_coordinators[f"{group}-{tree}"] = owner
            expected_leads[f"lf-{tree}"] = f"lead/lf/{tree}@hel8"
            expected_coordinators[f"lf-{tree}"] = "s/ladder@hel8"
        expected_leads["deploy"] = expected_coordinators["deploy"] = "s/deploy@hel8"
        self.assertEqual(leads, expected_leads)
        roster = pub.coordinators((DIRECTORY / "coordinators.txt").read_bytes())
        self.assertEqual(roster, expected_coordinators)
        for key, address in expected_coordinators.items():
            with self.subTest(key=key):
                self.assertEqual(pub.route(roster, f"ready/{key}/task"),
                                 {"route_status": "resolved", "coordinator": address})

    def test_new_route_parser_refuses_duplicate_third_column_and_bad_unassigned(self):
        for key in ("lf-temper", "lf-jig", "lf-skein", "lf-smith", "deploy"):
            with self.subTest(key=key):
                valid = f"{key} s/owner@fixture.example\n".encode()
                for invalid in (valid + valid, valid.rstrip() + b" third\n",
                                f"{key} unassigned\n".encode()):
                    with self.assertRaises(ValueError):
                        pub.coordinators(invalid)
                self.assertEqual(pub.route(pub.coordinators(f"{key} -\n".encode()), f"ready/{key}/task"),
                                 {"route_status": "unresolved", "route_reason": "unassigned"})


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
