// Packs the release kalk.exe into an MSIX for the Microsoft Store:
//   npx tauri build --no-bundle && node scripts/msix.mjs
// The Store signs the package after certification, so it stays unsigned here.
// The identity values come from Partner Center (Product identity) and must
// match exactly, or the upload is rejected.
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";

const IDENTITY_NAME = "AliBesinci.Kalk-SquatBreaks";
const PUBLISHER = "CN=03D962F2-BDB7-4A77-BE12-98DDFF678196";
const PUBLISHER_DISPLAY_NAME = "Ali Besinci";
// Must be a name reserved for the app in Partner Center.
const DISPLAY_NAME = "Kalk - Squat Breaks";

const { version } = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const icons = "src-tauri/icons";
const out = "src-tauri/target/msix";
const layout = join(out, "layout");
const exe = "src-tauri/target/release/kalk.exe";
if (!existsSync(exe)) {
  console.error(`${exe} is missing; run \`npx tauri build --no-bundle\` first`);
  process.exit(1);
}

rmSync(out, { recursive: true, force: true });
mkdirSync(join(layout, "Assets"), { recursive: true });
copyFileSync(exe, join(layout, "kalk.exe"));
for (const logo of ["StoreLogo", "Square44x44Logo", "Square150x150Logo"]) {
  copyFileSync(join(icons, `${logo}.png`), join(layout, "Assets", `${logo}.png`));
}

// The Store wants four version parts with the last one 0.
writeFileSync(
  join(layout, "AppxManifest.xml"),
  `<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap desktop rescap">
  <Identity Name="${IDENTITY_NAME}" Publisher="${PUBLISHER}" Version="${version}.0" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>${DISPLAY_NAME}</DisplayName>
    <PublisherDisplayName>${PUBLISHER_DISPLAY_NAME}</PublisherDisplayName>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-us" />
    <Resource Language="tr-tr" />
  </Resources>
  <Applications>
    <Application Id="Kalk" Executable="kalk.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="Kalk"
        Description="10 squats every 45 minutes"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\\Square150x150Logo.png"
        Square44x44Logo="Assets\\Square44x44Logo.png" />
      <Extensions>
        <!-- TaskId matches STARTUP_TASK in src-tauri/src/store.rs. -->
        <desktop:Extension Category="windows.startupTask" Executable="kalk.exe" EntryPoint="Windows.FullTrustApplication">
          <desktop:StartupTask TaskId="KalkStartup" Enabled="true" DisplayName="Kalk" />
        </desktop:Extension>
      </Extensions>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
`,
);

// makeappx ships with the Windows SDK; take the newest one installed.
const kits = "C:/Program Files (x86)/Windows Kits/10/bin";
const sdk = readdirSync(kits)
  .filter((v) => /^10\./.test(v) && existsSync(join(kits, v, "x64/makeappx.exe")))
  .sort((a, b) => a.localeCompare(b, undefined, { numeric: true }))
  .pop();
if (!sdk) {
  console.error("makeappx.exe not found; install the Windows SDK");
  process.exit(1);
}
const msix = join(out, `Kalk_${version}_x64.msix`);
execFileSync(join(kits, sdk, "x64/makeappx.exe"), ["pack", "/o", "/d", layout, "/p", msix], { stdio: "inherit" });
console.log(`packed ${msix}`);
