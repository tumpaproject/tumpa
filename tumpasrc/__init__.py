import io
import os
import sys
from PySide2 import QtWidgets
from PySide2.QtCore import QObject, Signal, QSize
from PySide2 import QtGui

import johnnycanencrypt as jce
import johnnycanencrypt.johnnycanencrypt as rjce


class NewKeyDialog(QtWidgets.QDialog):
    def __init__(self):
        super(NewKeyDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(QSize(800, 600))
        vboxlayout = QtWidgets.QVBoxLayout()
        name_label = QtWidgets.QLabel("Your name:")
        self.name_box = QtWidgets.QLineEdit("")

        vboxlayout.addWidget(name_label)
        vboxlayout.addWidget(self.name_box)

        email_label = QtWidgets.QLabel("Email addresses (one email per line)")
        self.email_box = QtWidgets.QPlainTextEdit()

        vboxlayout.addWidget(email_label)
        vboxlayout.addWidget(self.email_box)
        passphrase_label = QtWidgets.QLabel("Key Passphrase (must be 12+ chars in length):")
        self.passphrase_box = QtWidgets.QLineEdit("")

        vboxlayout.addWidget(passphrase_label)
        vboxlayout.addWidget(self.passphrase_box)

        self.generateButton = QtWidgets.QPushButton("Generateg New Key")
        self.generateButton.clicked.connect(self.generate)
        vboxlayout.addWidget(self.generateButton)

        self.setLayout(vboxlayout)
        self.setWindowTitle("Generate a new OpenPGP key")

    def generate(self):
        emails = self.email_box.toPlainText()
        name = self.name_box.text().strip()

        uids = []
        for email in emails.split("\n"):
            value = f"{name} <{email}>"
            uids.append(value)
        print(uids)
        self.hide()




class KeyWidget(QtWidgets.QWidget):
    selected = Signal((str),) 
    def __init__(self, key: jce.Key, selection_slot):
        super(KeyWidget, self).__init__()
        self.selected.connect(selection_slot)
        self.setMinimumWidth(400)
        self.key = key
        fingerprint = key.fingerprint
        self.fingerprint = fingerprint
        date = key.creationtime.date()
        self.keyfingerprint = QtWidgets.QLabel(fingerprint)
        date_label = QtWidgets.QLabel(date.strftime("%Y-%m-%d"))
        hlayout = QtWidgets.QHBoxLayout()
        hlayout.addWidget(self.keyfingerprint)
        hlayout.addWidget(date_label)
        fp_date_label = QtWidgets.QWidget()
        fp_date_label.setLayout(hlayout)
        group_vboxlayout = QtWidgets.QVBoxLayout()
        group_vboxlayout.addWidget(fp_date_label)
        print(key.uids)
        for uid in key.uids:
            uid_label = QtWidgets.QLabel(uid["value"])
            group_vboxlayout.addWidget(uid_label)
        self.setLayout(group_vboxlayout)

    def mousePressEvent(self, event):
        st = """QWidget {
            background: blue;
            color: white;
        }
        """
        self.setStyleSheet(st)
        self.selected.emit(self.fingerprint)

    def mouseDoubleClickEvent(self, event):
        select_path = QtWidgets.QFileDialog.getExistingDirectory(self, "Select directory to save public key", ".", QtWidgets.QFileDialog.ShowDirsOnly)
        if select_path:
            filepassphrase = f"{self.fingerprint}.pub"
            filepath = os.path.join(select_path, filepassphrase)
            with open(filepath, "w") as fobj:
                fobj.write(self.key.get_pub_key())
        print("public key written.")

    def deselected(self):
        st = """ QLabel {
            background: white;
            color: black;
            font: normal;
        }
        """
        self.setStyleSheet(st)

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, parent=None, config={}):
        super(MainWindow, self).__init__(parent)
        self.setWindowTitle("Tumpa")
        self.setMinimumWidth(500)
        self.ks = jce.KeyStore("./")
        self.key_widgets = []
        self.updateKeyList()
        self.current_fingerprint = ""

        self.cwidget = QtWidgets.QWidget()
        self.generateButton = QtWidgets.QPushButton(text="Generate new key")
        self.generateButton.clicked.connect(self.show_generate_dialog)
        self.uploadButton = QtWidgets.QPushButton(text="Upload to Yubikey")

        hlayout = QtWidgets.QHBoxLayout()
        hlayout.addWidget(self.generateButton)
        hlayout.addWidget(self.uploadButton)
        wd = QtWidgets.QWidget()
        wd.setLayout(hlayout)

        vboxlayout = QtWidgets.QVBoxLayout()
        vboxlayout.addWidget(self.widget)
        vboxlayout.addWidget(wd)
        self.cwidget.setLayout(vboxlayout)
        self.setCentralWidget(self.cwidget)

    def updateKeyList(self):
        vboxlayout = QtWidgets.QVBoxLayout()
        self.widget = QtWidgets.QWidget()
        for key in self.ks.get_all_keys():
            kw = KeyWidget(key, self.childSelected)
            self.key_widgets.append(kw)
            vboxlayout.addWidget(kw)
        # now set on the main window
        self.widget.setLayout(vboxlayout)

    def childSelected(self, fp):
        self.current_fingerprint = fp
        print(f"{fp=}")
        for kw in self.key_widgets:
            if kw.fingerprint != fp:
                kw.deselected()

    def show_generate_dialog(self):
        self.newd = NewKeyDialog()
        self.newd.show()


def main():
    app = QtWidgets.QApplication(sys.argv)
    form = MainWindow()
    form.show()
    app.exec_()

if __name__ == "__main__":
    main()
