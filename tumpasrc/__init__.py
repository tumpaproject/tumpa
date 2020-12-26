import io
import os
import sys
import datetime
from PySide2 import QtWidgets
from PySide2.QtCore import QObject, Signal, QSize
from PySide2 import QtGui

import johnnycanencrypt as jce
import johnnycanencrypt.johnnycanencrypt as rjce


class NewKeyDialog(QtWidgets.QDialog):
    update_ui = Signal((jce.Key,))

    def __init__(self, ks: jce.KeyStore, newkey_slot):
        super(NewKeyDialog, self).__init__()
        self.setModal(True)
        self.update_ui.connect(newkey_slot)
        self.ks = ks  # jce.KeyStore
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
        passphrase_label = QtWidgets.QLabel(
            "Key Passphrase (must be 12+ chars in length):"
        )
        self.passphrase_box = QtWidgets.QLineEdit("")

        vboxlayout.addWidget(passphrase_label)
        vboxlayout.addWidget(self.passphrase_box)

        self.generateButton = QtWidgets.QPushButton("Generateg New Key")
        self.generateButton.clicked.connect(self.generate)
        vboxlayout.addWidget(self.generateButton)

        self.setLayout(vboxlayout)
        self.setWindowTitle("Generate a new OpenPGP key")

    def generate(self):
        self.generateButton.setEnabled(False)
        emails = self.email_box.toPlainText()
        name = self.name_box.text().strip()
        password = self.passphrase_box.text().strip()

        uids = []
        for email in emails.split("\n"):
            value = f"{name} <{email}>"
            uids.append(value)
        edate = datetime.datetime.now() + datetime.timedelta(days=3 * 365)
        newk = self.ks.create_newkey(
            password,
            uids,
            ciphersuite=jce.Cipher.Cv25519,
            expiration=edate,
            subkeys_expiration=True,
        )
        self.update_ui.emit(newk)
        self.hide()


class KeyWidget(QtWidgets.QWidget):
    SPACER = 14
    BOTTOM_SPACER = 11

    def __init__(self, key: jce.Key):
        super(KeyWidget, self).__init__()
        self.setObjectName("KeyWidgetItem")
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
        group_vboxlayout.setSpacing(0)
        group_vboxlayout.setContentsMargins(0, 0, 0, 0)
        group_vboxlayout.addWidget(fp_date_label)
        for uid in key.uids:
            uid_label = QtWidgets.QLabel(uid["value"])
            group_vboxlayout.addWidget(uid_label)
        # group_vboxlayout.addItem(QtWidgets.QSpacerItem(self.BOTTOM_SPACER, self.BOTTOM_SPACER))
        self.setLayout(group_vboxlayout)
        self.setSizePolicy(QtWidgets.QSizePolicy.Fixed, QtWidgets.QSizePolicy.Fixed)

    def mouseDoubleClickEvent(self, event):
        select_path = QtWidgets.QFileDialog.getExistingDirectory(
            self,
            "Select directory to save public key",
            ".",
            QtWidgets.QFileDialog.ShowDirsOnly,
        )
        if select_path:
            filepassphrase = f"{self.fingerprint}.pub"
            filepath = os.path.join(select_path, filepassphrase)
            with open(filepath, "w") as fobj:
                fobj.write(self.key.get_pub_key())


class KeyWidgetList(QtWidgets.QListWidget):
    def __init__(self, ks):
        super(KeyWidgetList, self).__init__()
        self.setUniformItemSizes(True)
        self.setObjectName("KeyWidgetList")
        self.ks = ks

        self.key_widgets = []
        # Set layout.
        # self.layout = QtWidgets.QVBoxLayout(self)
        # self.setLayout(self.layout)
        self.updateList()
        self.setSizePolicy(QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Minimum)
        self.setMinimumHeight(500)
        self.currentItemChanged.connect(self.on_item_changed)

    def updateList(self):
        try:
            for key in self.ks.get_all_keys():
                kw = KeyWidget(key)
                item = QtWidgets.QListWidgetItem()
                item.setSizeHint(kw.sizeHint())
                self.addItem(item)
                self.setItemWidget(item, kw)
                self.key_widgets.append(kw)
        except:
            pass

    def on_item_changed(self):
        print(self.selectedItems())

    def addnewKey(self, key):
        kw = KeyWidget(key)
        item = QtWidgets.QListWidgetItem()
        item.setSizeHint(kw.sizeHint())
        self.addItem(item)
        self.setItemWidget(item, kw)
        self.key_widgets.append(kw)


css = """QPushButton {
    background-color: #3c99dc;
    border-style: outset;
    border-width: 2px;
    border-radius: 10px;
    border-color: beige;
    font: 14px;
    color: white;
    min-width: 10em;
    min-height: 40px;
    padding: 6px;
}
QPushButton:pressed {
    background-color: rgb(224, 0, 0);
    border-style: inset;
}
"""

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, parent=None, config={}):
        super(MainWindow, self).__init__(parent)
        self.setWindowTitle("Tumpa")
        self.setMinimumWidth(600)
        self.setMinimumHeight(500)
        self.ks = jce.KeyStore("./")
        self.vboxlayout_for_keys = QtWidgets.QVBoxLayout()
        self.widget = KeyWidgetList(self.ks)
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
        self.setStyleSheet(css)

    def show_generate_dialog(self):
        self.newd = NewKeyDialog(self.ks, self.widget.addnewKey)
        self.newd.show()



def main():
    app = QtWidgets.QApplication(sys.argv)
    form = MainWindow()
    form.show()
    app.exec_()


if __name__ == "__main__":
    main()
