#!/usr/bin/env python3
"""K-200: son Mach-O minos native C nesne başlığının yerine geçmez."""
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest
from unittest.mock import patch

API=runpy.run_path(str(Path(__file__).resolve().parents[1]/'native-deployment-denetle.py'))


class DeploymentTestleri(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='zee-native-deploy-')
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        self.obj=self.root/'release/build/ring-test/out/native.o'
        self.obj.parent.mkdir(parents=True)
        self.obj.write_bytes(b'object test bytes')

    def header(self,version):
        return ('Load command 0\n cmd LC_BUILD_VERSION\n platform 1\n minos '+version+'\n sdk 27.0\n').encode()

    def test_yeni_native_hedef_rust_esigini_asarsa_reddedilir(self):
        with patch('subprocess.run',return_value=subprocess.CompletedProcess([],0,self.header('27.0'),b'')):
            with self.assertRaisesRegex(ValueError,'exceeds Rust target'):
                API['denetle'](self.root,'11.0')

    def test_esit_hedef_hash_ve_baslikla_kanitlanir(self):
        with patch('subprocess.run',return_value=subprocess.CompletedProcess([],0,self.header('11.0.0'),b'')):
            report=API['denetle'](self.root,'11.0')
        self.assertEqual(report['objects'][0]['deployment_versions'],['11.0.0'])
        self.assertEqual(len(report['objects'][0]['sha256']),64)

    def test_kayip_surumu_baska_tool_version_kapatamaz(self):
        text=b'Load command 0\n cmd LC_SOURCE_VERSION\n version 11.0\n'
        with patch('subprocess.run',return_value=subprocess.CompletedProcess([],0,text,b'')):
            with self.assertRaisesRegex(ValueError,'evidence missing'):
                API['denetle'](self.root,'11.0')

    def test_ios_platformu_mac_kaniti_olamaz(self):
        with self.assertRaisesRegex(ValueError,'platform'):
            API['nesne_basligi'](self.header('11.0').decode().replace('platform 1','platform 2'))

    def test_okunamayan_nesne_ve_bozuk_surumu_reddeder(self):
        with patch('subprocess.run',return_value=subprocess.CompletedProcess([],1,b'',b'')):
            with self.assertRaisesRegex(ValueError,'unreadable'):
                API['denetle'](self.root,'11.0')
        with self.assertRaises(ValueError):API['surum']('11.0; run')


if __name__=='__main__':unittest.main()
