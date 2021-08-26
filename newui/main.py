# This Python file uses the following encoding: utf-8
import os
from pathlib import Path
import sys

from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine
from PySide6.QtCore import QObject, Slot, QUrl, Signal


class AdderModel(QObject):

    answerBack = Signal(int)

    def __init__(self):
        QObject.__init__(self)

    @Slot(int, int)
    def add(self, a, b):
        result = a + b
        print(f"Answer is {result}")
        self.answerBack.emit(result)


if __name__ == "__main__":
    app = QGuiApplication(sys.argv)
    url = os.fspath(Path(__file__).resolve().parent / "main.qml")
    print(url)
    engine = QQmlApplicationEngine()

    # Our model
    adder = AdderModel()
    ctxt = engine.rootContext()

    ctxt.setContextProperty("adderModel", adder)

    engine.load(QUrl.fromLocalFile(url))

    app.exec()
    sys.exit(0)
