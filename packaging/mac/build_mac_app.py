#!/usr/bin/env python3

import os
import inspect
import subprocess
import argparse
import shutil
import glob
import itertools

root = os.path.dirname(
    os.path.dirname(
        os.path.dirname(
            os.path.dirname(os.path.abspath(inspect.getfile(inspect.currentframe())))
        )
    )
)

root = os.path.join(root, "tumpa")


def codesign(path, entitlements, identity):
    run(
        [
            "codesign",
            "--sign",
            identity,
            "--entitlements",
            str(entitlements),
            "--timestamp",
            "--deep",
            str(path),
            "--force",
            "--options",
            "runtime",
        ]
    )


def run(cmd, cwd=None):
    subprocess.run(cmd, cwd=cwd, check=True)


def main():
    # Parse arguments
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--with-codesign",
        action="store_true",
        dest="with_codesign",
        help="Codesign the app bundle",
    )
    args = parser.parse_args()



    print("○ Clean up from last build")

    print(root)

    if os.path.exists(os.path.join(root, "macOS")):
        shutil.rmtree(os.path.join(root, "macOS/app/tumpa"))

    print("○ Building Tumpa")

    run(["briefcase", "create"])


    app_path = os.path.join(root, "macOS", "app", "tumpa", "tumpa.app")

    print("○ Delete unused Qt5 frameworks from app bundle")
    for framework in [
        "Qt3DAnimation",
        "Qt3DCore",
        "Qt3DExtras",
        "Qt3DInput",
        "Qt3DLogic",
        "Qt3DQuick",
        "Qt3DQuickAnimation",
        "Qt3DQuickExtras",
        "Qt3DQuickInput",
        "Qt3DQuickRender",
        "Qt3DQuickScene2D",
        "Qt3DRender",
        "QtBluetooth",
        "QtBodymovin",
        "QtCharts",
        "QtConcurrent",
        "QtDataVisualization",
        "QtDesigner",
        "QtDesignerComponents",
        "QtGamepad",
        "QtHelp",
        "QtLocation",
        "QtMultimedia",
        "QtMultimediaQuick",
        "QtMultimediaWidgets",
        "QtNfc",
        "QtOpenGL",
        "QtPdf",
        "QtPdfWidgets",
        "QtPositioning",
        "QtPositioningQuick",
        "QtPurchasing",
        "QtQuick",
        "QtQuick3D",
        "QtQuick3DAssetImport",
        "QtQuick3DRender",
        "QtQuick3DRuntimeRender",
        "QtQuick3DUtils",
        "QtQuickControls2",
        "QtQuickParticles",
        "QtQuickShapes",
        "QtQuickTemplates2",
        "QtQuickTest",
        "QtQuickWidgets",
        "QtRemoteObjects",
        "QtRepParser",
        "QtScript",
        "QtScriptTools",
        "QtScxml",
        "QtSensors",
        "QtSerialBus",
        "QtSerialPort",
        "QtSql",
        "QtTest",
        "QtTextToSpeech",
        "QtUiPlugin",
        "QtVirtualKeyboard",
        "QtWebChannel",
        "QtWebEngine",
        "QtWebEngineCore",
        "QtWebEngineWidgets",
        "QtWebSockets",
        "QtWebView",
        "QtXml",
        "QtXmlPatterns",
    ]:
        shutil.rmtree(
            os.path.join(
                app_path,
                "Contents",
                "Resources",
                "app_packages",
                "PySide2",
                "Qt",
                "lib",
                f"{framework}.framework",
            )
        )
        try:
            os.remove(
                os.path.join(
                    app_path,
                    "Contents",
                    "Resources",
                    "app_packages",
                    "PySide2",
                    f"{framework}.abi3.so",
                )
            )
            os.remove(
                os.path.join(
                    app_path,
                    "Contents",
                    "Resources",
                    "app_packages",
                    "PySide2",
                    f"{framework}.pyi",
                )
            )
        except FileNotFoundError:
            pass
    shutil.rmtree(
        os.path.join(
            app_path,
            "Contents",
            "Resources",
            "app_packages",
            "PySide2",
            "Designer.app",
        )
    )

    print(f"○ Unsigned app bundle: {app_path}")

    if args.with_codesign:
        identity_name_application = "Developer ID Application: Kushal Das (A7WGUTKMK6)"
        entitlements_plist_path = os.path.join(
            root, "packaging", "mac", "Entitlements.plist"
        )
        #run(["cp", entitlements_plist_path, os.path.join(root, "macOS/app/tumpa/")])

        print("○ Code sign app bundle")
        #run(["briefcase", "package"])
        for path in itertools.chain(
            glob.glob(
                f"{app_path}/Contents/Resources/app_packages/**/*.dylib", recursive=True
            ),
            glob.glob(
                f"{app_path}/Contents/Resources/app_packages/**/*.so", recursive=True
            ),
            glob.glob(
                f"{app_path}/Contents/Resources/Support/**/*.dylib", recursive=True
            ),
            glob.glob(f"{app_path}/Contents/Resources/Support/**/*.so", recursive=True),
            glob.glob(
                f"{app_path}/Contents/Resources/app_packages/PySide2/Qt/lib/**/Versions/5/*",
                recursive=True,
            ),
            [
                f"{app_path}/Contents/Resources/app_packages/PySide2/pyside2-lupdate",
                f"{app_path}/Contents/Resources/app_packages/PySide2/rcc",
                f"{app_path}/Contents/Resources/app_packages/PySide2/uic",
                app_path,
            ],
            glob.glob(f"{app_path}/Contents/Resources/app_packages/johnnycanencrypt/.dylibs/*.dylib")
        ):
            codesign(path, entitlements_plist_path, identity_name_application)
        codesign(app_path, entitlements_plist_path, identity_name_application)
        print(f"○ Signed app bundle: {app_path}")

        if not os.path.exists("/usr/local/bin/create-dmg"):
            print("○ Error: create-dmg is not installed")
            return

        print("○ Create DMG")
        dmg_path = os.path.join(root, "macOS", "tumpa-0.1.3.dmg")
        cmd = [
                "create-dmg",
                "--volname",
                "Tumpa",
                "--volicon",
                os.path.join(
                    root, "files", "in.kushaldas.Tumpa.icns"
                ),
                "--window-size",
                "600",
                "400",
                "--icon-size",
                "100",
                "--icon",
                "tumpa.app",
                "100",
                "150",
                "--hide-extension",
                "tumpa.app",
                "--app-drop-link",
                "300",
                "150",
                dmg_path,
                app_path,
                "--identity",
                identity_name_application,
            ]
        
        print("Running: {0}".format(" ".join(cmd)))
        run(cmd)

        print(f"○ Finished building DMG: {dmg_path}")


if __name__ == "__main__":
    main()
