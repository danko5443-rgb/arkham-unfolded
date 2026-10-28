"""Behavior checks for the native diagnostic command boundary (no desktop needed)."""

import io
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import native_verify as native


class Desktop:
    def __init__(self):
        self.root = (0, 0)
        self.focused = True
        self.valid_identity = True
        self.bounds = (1000, 500, 800, 600)
        self.mask = 0

    def verify(self):
        if not self.valid_identity or not self.focused:
            raise native.Refused("identity or focus changed")

    def focus(self):
        self.focused = True
        self.verify()

    def capture(self, path):
        path.write_bytes(b"independent client pixels")
        return {"path": str(path), "source": "test desktop"}

    def geometry(self):
        return self.bounds

    def pointer(self):
        x, y, _, _ = self.bounds
        return self.root, (self.root[0] - x, self.root[1] - y), self.mask


class Portal:
    size = (3072, 1728)

    def __init__(self, desktop):
        self.desktop = desktop
        self.events = []
        self.last_move = None
        self.lose_focus = False
        self.starts = 0
        self.closes = 0

    def start(self):
        self.starts += 1

    def close(self):
        self.closes += 1

    def move(self, x, y):
        self.last_move = (x, y)
        self.desktop.root = (round(100 + x * 1.6), round(60 + y * 1.6))
        if self.lose_focus:
            self.desktop.focused = False

    def button(self, pressed):
        self.events.append(pressed)


class NativeCommands(unittest.TestCase):
    def test_bad_identity_and_nonfinite_or_off_window_targets_never_press(self):
        desktop = Desktop()
        portal = Portal(desktop)
        controller = native.Controller(desktop, portal, pause=lambda _: None)
        controller.calibrate()
        for x, y in [(800, 250), (400, -1), (float('nan'), 0), (float('inf'), 0)]:
            with self.assertRaises(native.Refused):
                controller.click(x, y)
        desktop.valid_identity = False
        with self.assertRaises(native.Refused):
            controller.click(400, 250)
        self.assertEqual(portal.events, [])

    def test_capture_names_preserve_existing_evidence_and_stay_in_run_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'before.png').write_bytes(b'original')
            for name in ['before.png', '../outside.png', '/tmp/outside.png', 'bad.txt']:
                with self.assertRaises(native.Refused):
                    native.capture_path(root, name)
            self.assertEqual((root / 'before.png').read_bytes(), b'original')
            self.assertEqual(native.capture_path(root, 'after.png'), root / 'after.png')

    def test_one_grant_survives_refusal_and_closes_on_end_of_input(self):
        desktop = Desktop()
        portal = Portal(desktop)
        events = []
        with tempfile.TemporaryDirectory() as directory:
            native.run_session(desktop, portal, Path(directory), io.StringIO(
                '{"action":"click","x":-1,"y":250}\n'
                '{"action":"click","x":400,"y":250}\n'
            ), events.append, pause=lambda _: None)
        self.assertEqual((portal.starts, portal.closes), (1, 1))
        self.assertEqual(portal.events, [True, False])
        self.assertTrue(any(event['status'] == 'refused' for event in events))

    def test_startup_refusal_can_be_recalibrated_without_another_grant(self):
        portal = Portal(Desktop())
        portal.size = (0, 0)
        events = []

        def recovery():
            portal.size = (3072, 1728)
            yield '{"action":"calibrate"}'
            yield '{"action":"click","x":400,"y":250}'
            yield '{"action":"stop"}'

        with tempfile.TemporaryDirectory() as directory:
            native.run_session(portal.desktop, portal, Path(directory), recovery(),
                               events.append, pause=lambda _: None)
        self.assertEqual((portal.starts, portal.closes), (1, 1))
        self.assertEqual(portal.events, [True, False])
        self.assertEqual([e['status'] for e in events],
                         ['capture', 'refused', 'ready', 'clicked', 'session-closed'])

    def test_changed_mapping_geometry_or_held_button_never_press(self):
        for change in ('mapping', 'geometry', 'button'):
            with self.subTest(change=change):
                desktop = Desktop()
                portal = Portal(desktop)
                controller = native.Controller(desktop, portal, pause=lambda _: None)
                controller.calibrate()
                move = portal.move

                def changed_move(x, y):
                    move(x, y)
                    if change == 'mapping':
                        desktop.root = (desktop.root[0] + 30, desktop.root[1])
                    elif change == 'geometry':
                        desktop.bounds = (1000, 500, 801, 600)
                    else:
                        desktop.mask = 256

                portal.move = changed_move
                with self.assertRaises(native.Refused):
                    controller.click(400, 250)
                self.assertEqual(portal.events, [])

    def test_interrupted_press_releases_button_and_closes_grant(self):
        desktop = Desktop()
        portal = Portal(desktop)

        def interrupt_on_press(pressed):
            portal.events.append(pressed)
            if pressed:
                raise KeyboardInterrupt

        portal.button = interrupt_on_press
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(KeyboardInterrupt):
                native.run_session(desktop, portal, Path(directory),
                                   ['{"action":"click","x":400,"y":250}'],
                                   lambda _: None, pause=lambda _: None)
        self.assertEqual(portal.events, [True, False])
        self.assertEqual(portal.closes, 1)

    def test_partial_portal_start_is_closed(self):
        portal = Portal(Desktop())

        def failed_start():
            portal.starts += 1
            raise native.Refused('source selection failed after CreateSession')

        portal.start = failed_start
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(native.Refused):
                native.run_session(portal.desktop, portal, Path(directory), [], lambda _: None)
        self.assertEqual((portal.starts, portal.closes), (1, 1))

    def test_measured_mapping_reaches_client_target_and_refuses_lost_focus(self):
        desktop = Desktop()
        portal = Portal(desktop)
        controller = native.Controller(desktop, portal, pause=lambda _: None)
        controller.calibrate()
        controller.click(400, 250)
        self.assertAlmostEqual(desktop.pointer()[1][0], 400, delta=2)
        self.assertAlmostEqual(desktop.pointer()[1][1], 250, delta=2)
        self.assertEqual(portal.events, [True, False])

        portal.events.clear()
        portal.lose_focus = True
        with self.assertRaises(native.Refused):
            controller.click(400, 250)
        self.assertEqual(portal.events, [])


class DiagnosticBuild(unittest.TestCase):
    @unittest.skipUnless(shutil.which('cargo'), 'Cargo required for the real build-isolation check')
    def test_real_cargo_build_cannot_select_the_inherited_target_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            source.mkdir()
            (source / 'Cargo.toml').write_text(
                '[package]\nname="native-diagnostic-fixture"\nversion="0.1.0"\nedition="2024"\n'
                '[features]\ntabletop=[]\n[[bin]]\nname="arkham-tabletop"\npath="main.rs"\n'
            )
            (source / 'main.rs').write_text('fn main() { println!("isolated diagnostic"); }\n')
            subprocess.run(['cargo', 'generate-lockfile', '--offline', '--manifest-path',
                            str(source / 'Cargo.toml')], check=True, capture_output=True)
            normal = root / 'normal-target'
            (normal / 'debug').mkdir(parents=True)
            reference = normal / 'debug/arkham-tabletop'
            reference.write_bytes(b'preserved original executable')
            with patch.dict(os.environ, {'CARGO_TARGET_DIR': str(normal)}):
                result = native.build_diagnostic(source, root / 'diagnostic', reference)
            self.assertEqual(reference.read_bytes(), b'preserved original executable')
            self.assertEqual(subprocess.check_output([result['binary']], text=True).strip(),
                             'isolated diagnostic')
            with self.assertRaises(native.Refused):
                native.build_diagnostic(source, source / 'target/new', reference)

            (source / 'main.rs').write_text('this is not valid Rust\n')
            with self.assertRaises(subprocess.CalledProcessError):
                native.build_diagnostic(source, root / 'failed-build', reference)
            self.assertEqual(reference.read_bytes(), b'preserved original executable')


if __name__ == "__main__":
    unittest.main()
