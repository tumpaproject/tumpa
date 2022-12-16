# This Python file uses the following encoding: utf-8
import sys
from pathlib import Path
from typing import final

from PySide6.QtGui import QGuiApplication, QFontDatabase, QFont
from PySide6.QtQml import QQmlApplicationEngine

def main():
    app = QGuiApplication(sys.argv)
    # First set the font
    font_file = Path(__file__).resolve().parent / "Inter.ttf"
    font = QFontDatabase.addApplicationFont(str(font_file))
    font_list = QFontDatabase.applicationFontFamilies(font)
    final_font = font_list[0]
    QGuiApplication.setFont(QFont(final_font))

    # Now continue working on rest of the UI/Application
    engine = QQmlApplicationEngine()
    qml_file = Path(__file__).resolve().parent / "main.qml"
    engine.load(qml_file)
    if not engine.rootObjects():
        sys.exit(-1)
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
