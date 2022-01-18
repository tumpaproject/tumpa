#!/usr/bin/env python3

import argparse
import inspect
import os
import shutil
import subprocess

root = os.path.dirname(
    os.path.dirname(
        os.path.dirname(
            os.path.dirname(os.path.abspath(
                inspect.getfile(inspect.currentframe())))
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
            "--options=runtime,library",
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
                "PySide6",
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
                    "PySide6",
                    f"{framework}.abi3.so",
                )
            )
            os.remove(
                os.path.join(
                    app_path,
                    "Contents",
                    "Resources",
                    "app_packages",
                    "PySide6",
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
            "PySide6",
            "Designer.app",
        )
    )

    print(f"○ Unsigned app bundle: {app_path}")

    if args.with_codesign:
        identity_name_application = "Developer ID Application: Kushal Das (A7WGUTKMK6)"
        entitlements_plist_path = os.path.join(
            root, "packaging", "mac", "Entitlements.plist"
        )
        # run(["cp", entitlements_plist_path, os.path.join(root, "macOS/app/tumpa/")])

        print("○ Code sign app bundle")

        # Sign for briefcase package
        files = [
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtCore.framework/Versions/5/QtCore",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtDBus.framework/Versions/5/QtDBus",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtGui.framework/Versions/5/QtGui",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtMacExtras.framework/Versions/5/QtMacExtras",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtNetwork.framework/Versions/5/QtNetwork",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtNetworkAuth.framework/Versions/5/QtNetworkAuth",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtPrintSupport.framework/Versions/5/QtPrintSupport",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtQml.framework/Versions/5/QtQml",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtQmlModels.framework/Versions/5/QtQmlModels",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtQmlWorkerScript.framework/Versions/5/QtQmlWorkerScript",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtSvg.framework/Versions/5/QtSvg",
            "Contents/Resources/app_packages/PySide6/Qt/lib/QtWidgets.framework/Versions/5/QtWidgets",
            "Contents/Resources/app_packages/PySide6/pyside6-lupdate",
            "Contents/Resources/app_packages/PySide6/rcc",
            "Contents/Resources/app_packages/PySide6/uic",
        ]

        for f in files:
            try:
                codesign(
                    os.path.join(app_path, f),
                    entitlements_plist_path,
                    identity_name_application,
                )
            except FileNotFoundError:
                pass

        run(["briefcase", "package"])


if __name__ == "__main__":
    main()
