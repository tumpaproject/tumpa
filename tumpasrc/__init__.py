import io
import os
import sys
import datetime
from PySide2 import QtWidgets
from PySide2.QtCore import QObject, Signal, QSize
from PySide2 import QtGui

import johnnycanencrypt as jce
import johnnycanencrypt.johnnycanencrypt as rjce
from tumpasrc.resources import load_icon

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

QPushButton:disabled {
    background-color: #BEBEBE;
}
QLineEdit {
    height: 40px;
    margin: 0px 0px 0px 0px;
    padding-left: 5px;
    border-radius: 10px;
}

QLabel#keyring_label {
    font-size: 25px;
}

QPlainTextEdit {
    border-radius: 20px;
    background-color: palette(base);
    padding-left: 5px;
    padding-top: 5px;
}
"""


class PasswordEdit(QtWidgets.QLineEdit):
    """
    A LineEdit with icons to show/hide password entries
    """

    CSS = """QLineEdit {
        border-radius: 10px;
        height: 30px;
        margin: 0px 0px 0px 0px;
    }
    """

    def __init__(self, parent):
        self.parent = parent
        super().__init__(self.parent)

        # Set styles
        self.setStyleSheet(self.CSS)

        self.visibleIcon = load_icon("eye_visible.svg")
        self.hiddenIcon = load_icon("eye_hidden.svg")

        self.setEchoMode(QtWidgets.QLineEdit.Password)
        self.togglepasswordAction = self.addAction(
            self.visibleIcon, QtWidgets.QLineEdit.TrailingPosition
        )
        self.togglepasswordAction.triggered.connect(self.on_toggle_password_Action)
        self.password_shown = False

    def on_toggle_password_Action(self):
        if not self.password_shown:
            self.setEchoMode(QtWidgets.QLineEdit.Normal)
            self.password_shown = True
            self.togglepasswordAction.setIcon(self.hiddenIcon)
        else:
            self.setEchoMode(QtWidgets.QLineEdit.Password)
            self.password_shown = False
            self.togglepasswordAction.setIcon(self.visibleIcon)


class SmartCardConfirmationDialog(QtWidgets.QDialog):
    # passphrase, adminpin
    writetocard = Signal(
        (str, str),
    )

    def __init__(self, nextsteps_slot):
        super(SmartCardConfirmationDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(600, 200)
        self.setWindowTitle("Enter passphrase and pin for the smartcard")
        layout = QtWidgets.QFormLayout(self)
        label = QtWidgets.QLabel("Key passphrase")
        self.passphraseEdit = PasswordEdit(self)
        layout.addRow(label, self.passphraseEdit)
        label = QtWidgets.QLabel("Admin Pin")
        self.addminPinEdit = PasswordEdit(self)
        layout.addRow(label, self.addminPinEdit)
        widget = QtWidgets.QWidget()
        widget.setLayout(layout)
        # now the button
        self.finalButton = QtWidgets.QPushButton(text="Write to smartcard")
        self.finalButton.clicked.connect(self.getPassphrases)
        vboxlayout = QtWidgets.QVBoxLayout()
        vboxlayout.addWidget(widget)
        vboxlayout.addWidget(self.finalButton)
        self.setLayout(vboxlayout)
        self.writetocard.connect(nextsteps_slot)
        self.setStyleSheet(css)

    def getPassphrases(self):
        passphrase = self.passphraseEdit.text().strip()
        adminpin = self.addminPinEdit.text().strip()
        if len(adminpin) < 8:
            self.smallpin = QtWidgets.QMessageBox()
            self.smallpin.setText("Admin pin must be 8 character or more.")
            self.smallpin.setIcon(QtWidgets.QMessageBox.Critical)
            self.smallpin.setWindowTitle("Admin pin too small")
            self.smallpin.setStyleSheet(css)
            self.smallpin.show()
            return

        self.hide()
        self.writetocard.emit(passphrase, adminpin)


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

        self.generateButton = QtWidgets.QPushButton("Generate")
        self.generateButton.clicked.connect(self.generate)
        self.generateButton.setMaximumWidth(50)
        vboxlayout.addWidget(self.generateButton)

        self.setLayout(vboxlayout)
        self.setWindowTitle("Generate a new OpenPGP key")
        self.setStyleSheet(css)

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
        self.setLayout(group_vboxlayout)

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
        except Exception as e:
            print(e)

    def on_item_changed(self):
        print(self.selectedItems())

    def addnewKey(self, key):
        kw = KeyWidget(key)
        item = QtWidgets.QListWidgetItem()
        item.setSizeHint(kw.sizeHint())
        self.addItem(item)
        self.setItemWidget(item, kw)
        self.key_widgets.append(kw)


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
        self.uploadButton.clicked.connect(self.upload_to_smartcard)
        self.uploadButton.setEnabled(False)
        self.widget.itemSelectionChanged.connect(self.enable_upload)

        hlayout = QtWidgets.QHBoxLayout()
        hlayout.addWidget(self.generateButton)
        hlayout.addWidget(self.uploadButton)
        wd = QtWidgets.QWidget()
        wd.setLayout(hlayout)

        keyring_label = QtWidgets.QLabel("Available keys")
        keyring_label.setObjectName("keyring_label")
        vboxlayout = QtWidgets.QVBoxLayout()
        vboxlayout.addWidget(keyring_label)
        vboxlayout.addWidget(self.widget)
        vboxlayout.addWidget(wd)
        self.cwidget.setLayout(vboxlayout)
        self.setCentralWidget(self.cwidget)
        self.setStyleSheet(css)

    def enable_upload(self):
        self.uploadButton.setEnabled(True)

    def show_generate_dialog(self):
        self.newd = NewKeyDialog(self.ks, self.widget.addnewKey)
        self.newd.show()

    def upload_to_smartcard(self):
        # This means no key is selected on the list
        if not self.widget.selectedItems():
            self.select_first = QtWidgets.QMessageBox()
            self.select_first.setText("Please select a key from the list.")
            self.select_first.setIcon(QtWidgets.QMessageBox.Information)
            self.select_first.setWindowTitle("No selected key")
            self.select_first.setStyleSheet(css)
            self.select_first.show()
            return

        item = self.widget.selectedItems()[0]
        kw = self.widget.itemWidget(item)
        print(kw.key)
        self.sccd = SmartCardConfirmationDialog(self.get_pins_and_passphrase_and_write)
        self.sccd.show()

    def get_pins_and_passphrase_and_write(self, passphrase, adminpin):
        print(passphrase, adminpin)


def main():
    app = QtWidgets.QApplication(sys.argv)
    form = MainWindow()
    form.show()
    app.exec_()


if __name__ == "__main__":
    main()
