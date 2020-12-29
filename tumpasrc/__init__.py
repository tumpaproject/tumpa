import io
import os
import sys
import datetime
from PySide2 import QtWidgets
from PySide2.QtCore import QObject, Signal, QSize, Qt
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
    font: 18px;
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
    font-size: 24px;
}

QLabel#keyring_instruction {
    font-size: 12px;
}

QPlainTextEdit {
    border-radius: 20px;
    background-color: palette(base);
    padding-left: 5px;
    padding-top: 5px;
}


QLabel#keyfingerprint {
    font-size: 18px;
    font-weight: 600;
}

QListWidget::item {
    background-color: #F1F8FD;
    border: 1px solid #9DCCEE;
    margin: 4px;
}
QListWidget::item:selected {
    background-color: #9DCCEE;
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


class MessageDialogs:
    """
    A class that contains dialogue QMessageBoxes for success, error, etc.
    """

    @classmethod
    def success_dialog(cls, msg: str):
        success_dialog = QtWidgets.QMessageBox()
        success_dialog.setText(f"{msg}")
        success_dialog.setIcon(QtWidgets.QMessageBox.Information)
        success_dialog.setWindowTitle("Success")
        success_dialog.setStyleSheet(css)
        return success_dialog

    @classmethod
    def error_dialog(cls, where: str, msg: str):
        error_dialog = QtWidgets.QMessageBox()
        error_dialog.setText(msg)
        error_dialog.setIcon(QtWidgets.QMessageBox.Critical)
        error_dialog.setWindowTitle(f"Error during {where}")
        error_dialog.setStyleSheet(css)
        return error_dialog


class SmartCardConfirmationDialog(QtWidgets.QDialog):
    # passphrase, adminpin
    writetocard = Signal(
        (str, str),
    )

    def __init__(
        self,
        nextsteps_slot,
        title="Enter passphrase and pin for the smartcard",
        firstinput="Key passphrase",
    ):
        super(SmartCardConfirmationDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(600, 200)
        self.setWindowTitle(title)
        layout = QtWidgets.QFormLayout(self)
        label = QtWidgets.QLabel(firstinput)
        self.firstinput = firstinput
        self.passphraseEdit = PasswordEdit(self)
        layout.addRow(label, self.passphraseEdit)
        label = QtWidgets.QLabel("Current Admin Pin")
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
            self.error_dialog = MessageDialogs.error_dialog(
                "Editing smart card details", "Admin pin must be 8 character or more."
            )
            self.error_dialog.show()
            return
        if len(passphrase) < 6:
            self.error_dialog = MessageDialogs.error_dialog(
                "Editing smart card details",
                "{} must be 6 character or more.".format(self.firstinput),
            )
            self.error_dialog.show()
            return

        self.hide()
        self.writetocard.emit(passphrase, adminpin)


class SmartCardTextDialog(QtWidgets.QDialog):
    # Public URL and Name
    writetocard = Signal(
        (str, str),
    )

    CSS = """QLineEdit {
        border-radius: 10px;
        height: 30px;
        margin: 0px 0px 0px 0px;
    }
    """

    def __init__(
        self,
        nextsteps_slot,
        title="Enter public URL",
        textInput="Public URL",
    ):
        super(SmartCardTextDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(600, 200)
        self.setWindowTitle(title)
        layout = QtWidgets.QFormLayout(self)
        label = QtWidgets.QLabel(textInput)
        self.textInput = textInput
        self.textField = QtWidgets.QLineEdit("")
        self.textField.setStyleSheet(self.CSS)
        layout.addRow(label, self.textField)
        label = QtWidgets.QLabel("Admin Pin")
        self.adminPinEdit = PasswordEdit(self)
        layout.addRow(label, self.adminPinEdit)
        widget = QtWidgets.QWidget()
        widget.setLayout(layout)
        # now the button
        self.finalButton = QtWidgets.QPushButton(text="Write to smartcard")
        self.finalButton.clicked.connect(self.getTextValue)
        vboxlayout = QtWidgets.QVBoxLayout()
        vboxlayout.addWidget(widget)
        vboxlayout.addWidget(self.finalButton)
        self.setLayout(vboxlayout)
        self.writetocard.connect(nextsteps_slot)
        self.setStyleSheet(css)

    def getTextValue(self):
        text = self.textField.text().strip()
        adminpin = self.adminPinEdit.text().strip()
        if len(adminpin) < 8:
            self.error_dialog = MessageDialogs.error_dialog(
                "Editing smart card details", "Admin pin must be 8 character or more."
            )
            self.error_dialog.show()
            return
        if len(text) > 35:
            self.error_dialog = MessageDialogs.error_dialog(
                "Editing smart card details",
                "{} must be less than 35 characters.".format(self.textInput),
            )
            self.error_dialog.show()
            return
        if not len(text):
            self.error_dialog = MessageDialogs.error_dialog(
                "Editing smart card details",
                "{} cannot be blank.".format(self.textInput),
            )
            self.error_dialog.show()
            return

        self.hide()
        self.writetocard.emit(text, adminpin)


class NewKeyDialog(QtWidgets.QDialog):
    update_ui = Signal((jce.Key,))
    disable_button = Signal()
    enable_button = Signal()

    def __init__(self, ks: jce.KeyStore, newkey_slot, disable_slot, enable_slot):
        super(NewKeyDialog, self).__init__()
        self.setModal(True)
        self.update_ui.connect(newkey_slot)
        self.disable_button.connect(disable_slot)
        self.enable_button.connect(enable_slot)
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
            "Key Passphrase (recommended: 12+ chars in length):"
        )
        self.passphrase_box = PasswordEdit(self)

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

        if not len(name):
            self.error_dialog = MessageDialogs.error_dialog(
                "generating new key", "Name cannot be blank."
            )
            self.error_dialog.show()
            self.generateButton.setEnabled(True)
            return

        if not len(emails):
            self.error_dialog = MessageDialogs.error_dialog(
                "generating new key", "There must be at least one email."
            )
            self.error_dialog.show()
            self.generateButton.setEnabled(True)
            return

        if not len(password):
            self.error_dialog = MessageDialogs.error_dialog(
                "generating new key", "Key passphrase cannot be blank."
            )
            self.error_dialog.show()
            self.generateButton.setEnabled(True)
            return

        if len(password) < 6:
            self.error_dialog = MessageDialogs.error_dialog(
                "generating new key",
                "Key Passphrase must be at least 6 characters long.",
            )
            self.error_dialog.show()
            self.generateButton.setEnabled(True)
            return

        uids = []
        for email in emails.split("\n"):
            value = f"{name} <{email}>"
            uids.append(value)
        edate = datetime.datetime.now() + datetime.timedelta(days=3 * 365)
        self.disable_button.emit()
        # To make sure that the Generate button is disabled first
        self.generateButton.setEnabled(False)
        self.update()
        self.repaint()
        # Now let us try to create a key
        newk = self.ks.create_newkey(
            password,
            uids,
            ciphersuite=jce.Cipher.Cv25519,
            expiration=edate,
            subkeys_expiration=True,
        )
        self.update_ui.emit(newk)
        self.hide()
        self.enable_button.emit()
        self.success_dialog = MessageDialogs.success_dialog(
            "Generated keys successfully!"
        )
        self.success_dialog.show()


class KeyWidget(QtWidgets.QWidget):
    SPACER = 14
    BOTTOM_SPACER = 11

    def __init__(self, key: jce.Key):
        super(KeyWidget, self).__init__()
        self.setObjectName("KeyWidgetItem")
        self.setMinimumWidth(400)
        self.setMinimumHeight(84)
        self.key = key
        fingerprint = key.fingerprint
        self.fingerprint = fingerprint
        self.keyfingerprint = QtWidgets.QLabel(fingerprint)
        self.keyfingerprint.setObjectName("keyfingerprint")
        date = key.creationtime.date()
        date_label = QtWidgets.QLabel(f"Created at: {date.strftime('%Y-%m-%d')}")
        date_label.setAlignment(Qt.AlignTop)
        date_label.setContentsMargins(0, 0, 0, 0)

        # UIDs
        uid_vboxlayout = QtWidgets.QVBoxLayout()
        uid_vboxlayout.setSpacing(0)
        uid_vboxlayout.setContentsMargins(0, 0, 0, 0)
        for uid in key.uids:
            uid_label = QtWidgets.QLabel(uid["value"])
            uid_vboxlayout.addWidget(uid_label)
        uid_widget = QtWidgets.QWidget()
        uid_widget.setLayout(uid_vboxlayout)

        # UID and date layout
        hlayout = QtWidgets.QHBoxLayout()
        hlayout.addWidget(uid_widget)
        hlayout.addWidget(date_label)
        hlayout.setAlignment(Qt.AlignTop)
        hlayout.setContentsMargins(11, 0, 11, 11)
        group_widget = QtWidgets.QWidget()
        group_widget.setLayout(hlayout)

        fp_group_layout = QtWidgets.QVBoxLayout()
        fp_group_layout.addWidget(self.keyfingerprint)
        fp_group_layout.addWidget(group_widget)

        self.setLayout(fp_group_layout)
        self.setToolTip("Double click to export public key")
        self.setObjectName("keywidget")

    def mouseDoubleClickEvent(self, event):
        if self.export_public_key(self, self.fingerprint, self.key.get_pub_key()):
            self.success_dialog = MessageDialogs.success_dialog(
                "Exported public key successfully!"
            )
            self.success_dialog.show()

    @classmethod
    def export_public_key(cls, widget, fingerprint, public_key):
        select_path = QtWidgets.QFileDialog.getExistingDirectory(
            widget,
            "Select directory to save public key",
            ".",
            QtWidgets.QFileDialog.ShowDirsOnly,
        )
        if select_path:
            filepassphrase = f"{fingerprint}.pub"
            filepath = os.path.join(select_path, filepassphrase)
            with open(filepath, "w") as fobj:
                fobj.write(public_key)
            return True
        return False


class KeyWidgetList(QtWidgets.QListWidget):
    def __init__(self, ks):
        super(KeyWidgetList, self).__init__()
        self.setObjectName("KeyWidgetList")
        self.ks = ks

        # Set layout.
        # self.layout = QtWidgets.QVBoxLayout(self)
        # self.setLayout(self.layout)
        self.updateList()
        self.setSizePolicy(QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Minimum)
        self.setMinimumHeight(400)
        self.currentItemChanged.connect(self.on_item_changed)

    def updateList(self):
        try:
            keys = self.ks.get_all_keys()
            keys.sort(key=lambda x: x.creationtime, reverse=True)
            for key in keys:
                kw = KeyWidget(key)
                item = QtWidgets.QListWidgetItem()
                item.setSizeHint(kw.sizeHint())
                self.addItem(item)
                self.setItemWidget(item, kw)
        except Exception as e:
            print(e)

    def on_item_changed(self):
        print(self.selectedItems())

    def addnewKey(self, key):
        kw = KeyWidget(key)
        item = QtWidgets.QListWidgetItem()
        item.setSizeHint(kw.sizeHint())
        self.insertItem(0, item)
        self.setItemWidget(item, kw)


class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, parent=None, config={}):
        super(MainWindow, self).__init__(parent)
        self.setWindowTitle("Tumpa")
        self.setMinimumWidth(600)
        self.setMinimumHeight(575)
        self.setMaximumWidth(600)
        self.setMaximumHeight(575)
        self.ks = jce.KeyStore("./")
        self.vboxlayout_for_keys = QtWidgets.QVBoxLayout()
        self.widget = KeyWidgetList(self.ks)
        self.current_fingerprint = ""

        # File menu
        exportPubKey = QtWidgets.QAction("&Export public key", self)
        exportPubKey.triggered.connect(self.export_public_key)
        exitAction = QtWidgets.QAction("E&xit", self)
        exitAction.triggered.connect(self.exit_process)
        menu = self.menuBar()
        filemenu = menu.addMenu("&File")
        filemenu.addAction(exportPubKey)
        filemenu.addAction(exitAction)

        # smartcard menu
        changepinAction = QtWidgets.QAction("Change user &pin", self)
        changepinAction.triggered.connect(self.show_change_user_pin_dialog)
        changeadminpinAction = QtWidgets.QAction("Change &admin pin", self)
        changeadminpinAction.triggered.connect(self.show_change_admin_pin_dialog)
        changenameAction = QtWidgets.QAction("Set cardholder &name", self)
        changenameAction.triggered.connect(self.show_set_name)
        changeurlAction = QtWidgets.QAction("Set public key &URL", self)
        changeurlAction.triggered.connect(self.show_set_public_url)
        resetYubiKeylAction = QtWidgets.QAction("Reset the YubiKey", self)
        resetYubiKeylAction.triggered.connect(self.reset_yubikey_dialog)
        smartcardmenu = menu.addMenu("&SmartCard")
        smartcardmenu.addAction(changepinAction)
        smartcardmenu.addAction(changeadminpinAction)
        smartcardmenu.addAction(changenameAction)
        smartcardmenu.addAction(changeurlAction)
        smartcardmenu.addAction(resetYubiKeylAction)

        self.cwidget = QtWidgets.QWidget()
        self.generateButton = QtWidgets.QPushButton(text="Generate new key")
        self.generateButton.clicked.connect(self.show_generate_dialog)
        self.uploadButton = QtWidgets.QPushButton(text="Upload to SmartCard")
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
        keyring_instruction_label = QtWidgets.QLabel(
            "Single click on a key to enable writing to smart card. "
            + "Double click on a key to export the public key."
        )
        keyring_instruction_label.setObjectName("keyring_instruction")
        vboxlayout = QtWidgets.QVBoxLayout()
        vboxlayout.addWidget(keyring_label)
        vboxlayout.addWidget(keyring_instruction_label)
        vboxlayout.addWidget(self.widget)
        vboxlayout.addWidget(wd)
        self.cwidget.setLayout(vboxlayout)
        self.setCentralWidget(self.cwidget)
        self.setStyleSheet(css)

    def reset_yubikey_dialog(self):
        "Verify if the user really wants to reset the smartcard"
        reply = QtWidgets.QMessageBox.question(
            self,
            "Are you sure?",
            "This action will reset your YubiKey. Are you sure to do that?",
        )
        if reply == QtWidgets.QMessageBox.StandardButton.Yes:
            try:
                rjce.reset_yubikey()
            except Exception as e:
                self.error_dialog = MessageDialogs.error_dialog(
                    "YubiKey reset.", str(e)
                )
                self.error_dialog.show()
                return
        else:
            return

        self.success_dialog = MessageDialogs.success_dialog(
            "YubiKey successfully reset."
        )
        self.success_dialog.show()

    def enable_upload(self):
        "Slot to enable the upload to smartcard button"
        self.uploadButton.setEnabled(True)

    def show_change_user_pin_dialog(self):
        "This slot shows the input dialog to change user pin"
        self.smalldialog = SmartCardConfirmationDialog(
            self.change_pin_on_card_slot, "Change user pin", "New User pin"
        )
        self.smalldialog.show()

    def show_set_public_url(self):
        "This slot shows the input dialog to set public url"
        self.smalldialog = SmartCardTextDialog(
            self.set_url_on_card_slot, "Add public URL", "Public URL"
        )
        self.smalldialog.show()

    def show_set_name(self):
        "This slot shows the input dialog to set name"
        self.smalldialog = SmartCardTextDialog(
            self.set_name_on_card_slot, "Add Name", "Name"
        )
        self.smalldialog.show()

    def show_change_admin_pin_dialog(self):
        "This slot shows the input dialog to change admin pin"
        self.smalldialog = SmartCardConfirmationDialog(
            self.change_admin_pin_on_card_slot, "Change admin pin", "New Admin pin"
        )
        self.smalldialog.show()

    def change_pin_on_card_slot(self, userpin, adminpin):
        "Final slot which will try to change the userpin"
        try:
            rjce.change_user_pin(adminpin.encode("utf-8"), userpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog("changing user pin", str(e))
            self.error_dialog.show()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Changed user pin successfully."
        )
        self.success_dialog.show()

    def change_admin_pin_on_card_slot(self, userpin, adminpin):
        "Final slot which will try to change the adminpin"
        try:
            rjce.change_admin_pin(adminpin.encode("utf-8"), userpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog(
                "changing admin pin", str(e)
            )
            self.error_dialog.show()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Changed admin pin successfully."
        )
        self.success_dialog.show()

    def set_url_on_card_slot(self, publicURL, adminpin):
        "Final slot which will try to change the publicURL"
        try:
            rjce.set_url(publicURL.encode("utf-8"), adminpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog("adding public URL", str(e))
            self.error_dialog.show()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Added public URL successfully."
        )
        self.success_dialog.show()

    def set_name_on_card_slot(self, name, adminpin):
        "Final slot which will try to change the name"
        try:
            # If input is "First Middle Last",
            # the parameter sent should be "Last<<Middle<<First"
            name = "<<".join(name.split()[::-1])
            rjce.set_name(name.encode("utf-8"), adminpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog("adding name", str(e))
            self.error_dialog.show()
            return
        self.success_dialog = MessageDialogs.success_dialog("Added name successfully.")
        self.success_dialog.show()

    def show_generate_dialog(self):
        "Shows the dialog to generate new key"
        self.newd = NewKeyDialog(
            self.ks,
            self.widget.addnewKey,
            self.disable_generate_button,
            self.enable_generate_button,
        )
        self.newd.show()

    def disable_generate_button(self):
        self.generateButton.setEnabled(False)
        self.update()
        self.repaint()

    def enable_generate_button(self):
        self.generateButton.setEnabled(True)
        self.update()
        self.repaint()

    def upload_to_smartcard(self):
        "Shows the userinput dialog to upload the selected key to the smartcard"
        # This means no key is selected on the list
        if not self.widget.selectedItems():
            self.error_dialog = MessageDialogs.error_dialog(
                "upload to smart card", "Please select a key from the list."
            )
            self.error_dialog.show()
            return

        item = self.widget.selectedItems()[0]
        kw = self.widget.itemWidget(item)
        self.current_key = kw.key
        self.sccd = SmartCardConfirmationDialog(self.get_pins_and_passphrase_and_write)
        self.sccd.show()

    def get_pins_and_passphrase_and_write(self, passphrase, adminpin):
        "This method uploads the cert to the card"
        print(passphrase, adminpin)
        certdata = self.current_key.keyvalue
        try:
            rjce.upload_to_smartcard(certdata, adminpin.encode("utf-8"), passphrase)
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog(
                "upload to smartcard.", str(e)
            )
            self.error_dialog.show()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Uploaded to the smartcard successfully."
        )
        self.success_dialog.show()

    def export_public_key(self):
        # This means no key is selected on the list
        if not self.widget.selectedItems():
            self.error_dialog = MessageDialogs.error_dialog(
                "exporting public key", "Please select a key from the list."
            )
            self.error_dialog.show()
            return

        item = self.widget.selectedItems()[0]
        kw = self.widget.itemWidget(item)
        if KeyWidget.export_public_key(self, kw.key.fingerprint, kw.key.get_pub_key()):
            self.success_dialog = MessageDialogs.success_dialog(
                "Exported public key successfully!"
            )
            self.success_dialog.show()

    def exit_process(self):
        sys.exit(0)


def main():
    app = QtWidgets.QApplication(sys.argv)
    form = MainWindow()
    form.show()
    app.exec_()


if __name__ == "__main__":
    main()
