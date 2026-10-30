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


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
