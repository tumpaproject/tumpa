import datetime
import io
import os
import sys
import time

import johnnycanencrypt as jce
import johnnycanencrypt.johnnycanencrypt as rjce
from tumpa.resources import load_icon, load_css
from tumpa.configuration import get_keystore_directory

css = load_css("mainwindow.css")
from PySide2 import QtGui, QtWidgets
from PySide2.QtCore import QObject, QSize, Qt, QThread, Signal

import tumpasrc.key_widgets.utils as key_utils
from tumpasrc.commons import MessageDialogs, PasswordEdit, css
from tumpasrc.configuration import get_keystore_directory
from tumpasrc.key_widgets.display import KeyWidgetList
from tumpasrc.key_widgets.forms import NewKeyFormWidget
from tumpasrc.resources import load_css, load_icon
from tumpasrc.smartcard_widgets.forms import (SmartCardConfirmationDialog,
                                              SmartCardTextFormWidget,
                                              SmartPinFormWidget)
from tumpasrc.threads import HardwareThread


class KeyViewWidget(QtWidgets.QWidget):
    def __init__(self, ks, parent=None):
        super(KeyViewWidget, self).__init__(parent)
        keyring_instruction_label = QtWidgets.QLabel(
            "Single click on a key to enable writing to smart card. "
            + "Double click on a key to export the public key."
        )
        keyring_instruction_label.setObjectName("keyring_instruction")

        # List of keys shown
        self.widget = KeyWidgetList(ks)

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
        (str, str, int),
    )

    def __init__(
        self,
        nextsteps_slot,
        title="Enter passphrase and pin for the smartcard",
        firstinput="Key passphrase",
        key=None,
        enable_window=None,
    ):
        super(SmartCardConfirmationDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(600, 220)
        self.setWindowTitle(title)
        if enable_window:
            self.rejected.connect(enable_window)
        layout = QtWidgets.QFormLayout(self)
        label = QtWidgets.QLabel(firstinput)
        self.firstinput = firstinput
        self.key = key
        self.encryptionSubkey = QtWidgets.QCheckBox("Encryption")
        self.encryptionSubkey.setEnabled(False)
        self.signingSubkey = QtWidgets.QCheckBox("Signing")
        self.signingSubkey.setEnabled(False)
        self.authenticationSubkey = QtWidgets.QCheckBox("Authentication")
        self.authenticationSubkey.setEnabled(False)
        self.passphraseEdit = PasswordEdit()
        layout.addRow(label, self.passphraseEdit)
        label = QtWidgets.QLabel("Current Admin Pin")
        self.addminPinEdit = PasswordEdit()
        layout.addRow(label, self.addminPinEdit)
        if self.key is not None:
            label = QtWidgets.QLabel("Choose subkeys to upload:")
            inhlayout = QtWidgets.QHBoxLayout()
            got_enc, got_sign, got_auth = self.key.available_subkeys()
            inhlayout.addWidget(self.encryptionSubkey)
            inhlayout.addWidget(self.signingSubkey)
            inhlayout.addWidget(self.authenticationSubkey)

            if got_enc:
                self.encryptionSubkey.setCheckState(Qt.Checked)
                self.encryptionSubkey.setEnabled(True)
            if got_sign:
                self.signingSubkey.setCheckState(Qt.Checked)
                self.signingSubkey.setEnabled(True)
            if got_auth:
                self.authenticationSubkey.setCheckState(Qt.Checked)
                self.authenticationSubkey.setEnabled(True)
            if any([got_enc, got_auth, got_sign]):  # Means we have at least one subkey
                widget = QtWidgets.QWidget()
                widget.setLayout(inhlayout)
                # Now add in the formlayout
                layout.addRow(label, widget)
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

        whichkeys = 0
        if self.encryptionSubkey.checkState():
            whichkeys += 1
        if self.signingSubkey.checkState():
            whichkeys += 2
        if self.authenticationSubkey.checkState():
            whichkeys += 4

        # At least one subkey must be selected
        if whichkeys == 0:
            self.error_dialog = MessageDialogs.error_dialog(
                "Editing smart card details", "At least one subkey must be selected"
            )
            self.error_dialog.show()
            return

        self.hide()

        self.writetocard.emit(passphrase, adminpin, whichkeys)


class SmartPinDialog(QtWidgets.QDialog):
    # passphrase, adminpin
    writetocard = Signal(
        (str, str),
    )

    def __init__(
        self,
        nextsteps_slot,
        title="Change user pin",
        firstinput="New user pin",
        enable_window=None,
    ):
        super(SmartPinDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(600, 220)
        self.setWindowTitle(title)
        if enable_window:
            self.rejected.connect(enable_window)
        layout = QtWidgets.QFormLayout(self)
        label = QtWidgets.QLabel(firstinput)
        self.firstinput = firstinput
        self.passphraseEdit = PasswordEdit()
        layout.addRow(label, self.passphraseEdit)
        label = QtWidgets.QLabel("Current Admin Pin")
        self.addminPinEdit = PasswordEdit()
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
        if self.firstinput == "New Admin pin" and len(passphrase) < 8:
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
        enable_window=None,
    ):
        super(SmartCardTextDialog, self).__init__()
        self.setModal(True)
        self.setFixedSize(600, 200)
        self.setWindowTitle(title)
        if enable_window:
            self.rejected.connect(enable_window)
        layout = QtWidgets.QFormLayout(self)
        label = QtWidgets.QLabel(textInput)
        self.textInput = textInput
        self.textField = QtWidgets.QLineEdit("")
        self.textField.setStyleSheet(self.CSS)
        layout.addRow(label, self.textField)
        label = QtWidgets.QLabel("Admin Pin")
        self.adminPinEdit = PasswordEdit()
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

    def __init__(
        self,
        ks: jce.KeyStore,
        newkey_slot,
        disable_slot,
        enable_slot,
        enable_window=None,
    ):
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
        if enable_window:
            self.rejected.connect(enable_window)

        vboxlayout.addWidget(name_label)
        vboxlayout.addWidget(self.name_box)

        email_label = QtWidgets.QLabel("Email addresses (one email per line)")
        self.email_box = QtWidgets.QPlainTextEdit()
        self.email_box.setTabChangesFocus(True)

        vboxlayout.addWidget(email_label)
        vboxlayout.addWidget(self.email_box)
        passphrase_label = QtWidgets.QLabel(
            "Key Passphrase (recommended: 12+ chars in length):"
        )
        self.passphrase_box = PasswordEdit()

        vboxlayout.addWidget(passphrase_label)
        vboxlayout.addWidget(self.passphrase_box)

        # now the checkboxes for subkey
        self.encryptionSubkey = QtWidgets.QCheckBox("Encryption subkey")
        self.encryptionSubkey.setCheckState(Qt.Checked)
        self.signingSubkey = QtWidgets.QCheckBox("Signing subkey")
        self.signingSubkey.setCheckState(Qt.Checked)
        self.authenticationSubkey = QtWidgets.QCheckBox("Authentication subkey")

        hboxlayout = QtWidgets.QHBoxLayout()
        hboxlayout.addWidget(self.encryptionSubkey)
        hboxlayout.addWidget(self.signingSubkey)
        hboxlayout.addWidget(self.authenticationSubkey)

        widget = QtWidgets.QWidget()
        widget.setLayout(hboxlayout)
        vboxlayout.addWidget(widget)

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

        # Now check which all subkeys are required
        whichkeys = 0
        if self.encryptionSubkey.checkState():
            whichkeys += 1
        if self.signingSubkey.checkState():
            whichkeys += 2
        if self.authenticationSubkey.checkState():
            whichkeys += 4

        # At least one subkey must be selected
        if whichkeys == 0:
            self.error_dialog = MessageDialogs.error_dialog(
                "Generating new key", "At least one subkey must be selected"
            )
            self.error_dialog.show()
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
            ciphersuite=jce.Cipher.RSA4k,
            expiration=edate,
            subkeys_expiration=True,
            whichkeys=whichkeys,
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
=======
>>>>>>> 25c721c (Refactor UI structure)
        # Buttons
        self.generateButton = QtWidgets.QPushButton(text="Generate new key")
        self.generateButton.clicked.connect(self.parent().show_generate_dialog)
        self.uploadButton = QtWidgets.QPushButton(text="Upload to SmartCard")
        self.uploadButton.clicked.connect(self.parent().upload_to_smartcard)
        self.uploadButton.setEnabled(False)
<<<<<<< HEAD
=======
>>>>>>> 9c4a15f (Refactor UI structure):tumpasrc/__init__.py
>>>>>>> 25c721c (Refactor UI structure)
        hlayout = QtWidgets.QHBoxLayout()
        hlayout.addWidget(self.generateButton)
        hlayout.addWidget(self.uploadButton)
        wd = QtWidgets.QWidget()
        wd.setLayout(hlayout)

        vboxlayout = QtWidgets.QVBoxLayout()
        vboxlayout.addWidget(keyring_instruction_label)
        vboxlayout.addWidget(self.widget)
        vboxlayout.addWidget(wd)
        self.setLayout(vboxlayout)


class SmartCardViewWidget(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super(SmartCardViewWidget, self).__init__(parent)
        self.main_area = SmartCardTextFormWidget(
            self.parent().set_name_on_card_slot,
            "Add Name",
            "Name",
        )
        self.main_area.setObjectName("mainarea")

        # Left Navbar
        self.editNameButton = QtWidgets.QPushButton(text="Edit Name")
        self.editNameButton.clicked.connect(self.parent().show_set_name)
        self.editNameButton.setProperty("active", "True")
        self.editPublicUrlButton = QtWidgets.QPushButton(text="Edit Public URL")
        self.editPublicUrlButton.clicked.connect(self.parent().show_set_public_url)
        self.editUserPinButton = QtWidgets.QPushButton(text="Edit User Pin")
        self.editUserPinButton.clicked.connect(self.parent().show_change_user_pin)
        self.editAdminPinButton = QtWidgets.QPushButton(text="Edit Admin Pin")
        self.editAdminPinButton.clicked.connect(self.parent().show_change_admin_pin)
        vnavlayout = QtWidgets.QVBoxLayout()
        vnavlayout.addWidget(self.editNameButton)
        vnavlayout.addWidget(self.editPublicUrlButton)
        vnavlayout.addWidget(self.editUserPinButton)
        vnavlayout.addWidget(self.editAdminPinButton)
        vnavlayout.setAlignment(Qt.AlignTop)
        navbarWidget = QtWidgets.QWidget()
        navbarWidget.setLayout(vnavlayout)
        navbarWidget.setObjectName("sidenavbar")
        navbarWidget.setMaximumWidth(200)

        self.hlayout = QtWidgets.QHBoxLayout()
        self.hlayout.addWidget(navbarWidget)
        self.hlayout.addWidget(self.main_area)
        self.hlayout.setMargin(0)
class SmartCardViewWidget(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super(SmartCardViewWidget, self).__init__(parent)
        self.main_area = QtWidgets.QLabel("This is where smart card stuff go.")

        # Left Navbar
        self.editNameButton = QtWidgets.QPushButton(text="Edit Name")
        self.editNameButton.clicked.connect(self.parent().show_set_name)
        self.editPublicUrlButton = QtWidgets.QPushButton(text="Edit Public URL")
        self.editPublicUrlButton.clicked.connect(self.parent().show_set_public_url)
        self.editUserPinButton = QtWidgets.QPushButton(text="Edit User Pin")
        self.editUserPinButton.clicked.connect(self.parent().show_change_user_pin)
        self.editAdminPinButton = QtWidgets.QPushButton(text="Edit Admin Pin")
        self.editAdminPinButton.clicked.connect(self.parent().show_change_admin_pin)
        vnavlayout = QtWidgets.QVBoxLayout()
        vnavlayout.addWidget(self.editNameButton)
        vnavlayout.addWidget(self.editPublicUrlButton)
        vnavlayout.addWidget(self.editUserPinButton)
        vnavlayout.addWidget(self.editAdminPinButton)
        navbarWidget = QtWidgets.QWidget()
        navbarWidget.setLayout(vnavlayout)
        navbarWidget.setObjectName("sidenavbar")
        navbarWidget.setMaximumWidth(180)

        self.hlayout = QtWidgets.QHBoxLayout()
        self.hlayout.addWidget(navbarWidget)
        self.hlayout.addWidget(self.main_area)
        self.setLayout(self.hlayout)


class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, parent=None, config={}):
        super(MainWindow, self).__init__(parent)
        self.setWindowTitle("Tumpa: OpenPGP made simple")
        self.setMinimumWidth(600)
        self.setMinimumHeight(575)
        self.setMaximumWidth(600)
        self.setMaximumHeight(575)
        self.ks = jce.KeyStore(get_keystore_directory())
        self.current_fingerprint = ""
        self.cardcheck_thread = HardwareThread(
            self.enable_upload, self.update_statusbar)
        self.statusBar = QtWidgets.QStatusBar()
        self.setStatusBar(self.statusBar)

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
        changepinAction.triggered.connect(self.show_change_user_pin)
        changeadminpinAction = QtWidgets.QAction("Change &admin pin", self)
        changeadminpinAction.triggered.connect(self.show_change_admin_pin)
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

        # self.widget.itemSelectionChanged.connect(self.enable_upload)
        self.tabs = QtWidgets.QTabWidget()
        self.keyWidget = KeyViewWidget(self.ks, self)
        self.tabs.addTab(self.keyWidget, "Available Keys")
        self.smartCardWidget = SmartCardViewWidget(self)
        self.tabs.addTab(self.smartCardWidget, "Smart Card Settings")
        self.setCentralWidget(self.tabs)
        self.setStyleSheet(css)
        self.cardcheck_thread.start()

    def update_statusbar(self, value):
        self.statusBar.showMessage(value)

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

    def enable_upload(self, value):
        "Slot to enable the upload to smartcard button"
        # If no item is selected on the ListWidget, then
        # no need to update the uploadButton status.
        try:
            if not self.keyWidget.widget.selectedItems():
                return
        except RuntimeError:
            return
        self.keyWidget.uploadButton.setEnabled(value)

    def show_change_user_pin(self):
        "This slot shows the input widget to change user pin"
        self.cardcheck_thread.flag = False
        self.tabs.setCurrentIndex(1)
        self.activateSmartCardOption("userPin")
        self.smartCardWidget.hlayout.removeWidget(self.smartCardWidget.main_area)
        self.smartCardWidget.main_area.deleteLater()
        self.smartCardWidget.main_area = SmartPinFormWidget(
            self.change_pin_on_card_slot,
            "Change user pin",
            "New User pin",
        )
        self.smartCardWidget.hlayout.addWidget(self.smartCardWidget.main_area)

    def show_set_public_url(self):
        "This slot shows the input widget to set public url"
        self.cardcheck_thread.flag = False
        self.tabs.setCurrentIndex(1)
        self.activateSmartCardOption("url")
        self.smartCardWidget.hlayout.removeWidget(self.smartCardWidget.main_area)
        self.smartCardWidget.main_area.deleteLater()
        self.smartCardWidget.main_area = SmartCardTextFormWidget(
            self.set_url_on_card_slot,
            "Add public URL",
            "Public URL",
        )
        self.smartCardWidget.hlayout.addWidget(self.smartCardWidget.main_area)

    def show_set_name(self):
        "This slot shows the input widget to set name"
        self.cardcheck_thread.flag = False
        self.tabs.setCurrentIndex(1)
        self.activateSmartCardOption("name")
        self.smartCardWidget.hlayout.removeWidget(self.smartCardWidget.main_area)
        self.smartCardWidget.main_area.deleteLater()
        self.smartCardWidget.main_area = SmartCardTextFormWidget(
            self.set_name_on_card_slot,
            "Add Name",
            "Name",
        )
        self.smartCardWidget.hlayout.addWidget(self.smartCardWidget.main_area)

    def show_change_admin_pin(self):
        "This slot shows the input widget to change admin pin"
        self.cardcheck_thread.flag = False
        self.tabs.setCurrentIndex(1)
        self.activateSmartCardOption("adminPin")
        self.smartCardWidget.hlayout.removeWidget(self.smartCardWidget.main_area)
        self.smartCardWidget.main_area.deleteLater()
        self.smartCardWidget.main_area = SmartPinFormWidget(
            self.change_admin_pin_on_card_slot,
            "Change admin pin",
            "New Admin pin",
        )
        self.smartCardWidget.hlayout.addWidget(self.smartCardWidget.main_area)

    def activateSmartCardOption(self, buttonName: str):
        self.deactivateAllSmartCardOption()
        if buttonName == "adminPin":
            self.smartCardWidget.editAdminPinButton.setProperty("active", "True")
            self.smartCardWidget.editAdminPinButton.style().unpolish(
                self.smartCardWidget.editAdminPinButton
            )
            self.smartCardWidget.editAdminPinButton.style().polish(
                self.smartCardWidget.editAdminPinButton
            )
        elif buttonName == "userPin":
            self.smartCardWidget.editUserPinButton.setProperty("active", "True")
            self.smartCardWidget.editUserPinButton.style().unpolish(
                self.smartCardWidget.editUserPinButton
            )
            self.smartCardWidget.editUserPinButton.style().polish(
                self.smartCardWidget.editUserPinButton
            )
        elif buttonName == "url":
            self.smartCardWidget.editPublicUrlButton.setProperty("active", "True")
            self.smartCardWidget.editPublicUrlButton.style().unpolish(
                self.smartCardWidget.editPublicUrlButton
            )
            self.smartCardWidget.editPublicUrlButton.style().polish(
                self.smartCardWidget.editPublicUrlButton
            )
        elif buttonName == "name":
            self.smartCardWidget.editNameButton.setProperty("active", "True")
            self.smartCardWidget.editNameButton.style().unpolish(
                self.smartCardWidget.editNameButton
            )
            self.smartCardWidget.editNameButton.style().polish(
                self.smartCardWidget.editNameButton
            )

    def deactivateAllSmartCardOption(self):
        self.smartCardWidget.editNameButton.setProperty("active", "False")
        self.smartCardWidget.editPublicUrlButton.setProperty("active", "False")
        self.smartCardWidget.editUserPinButton.setProperty("active", "False")
        self.smartCardWidget.editAdminPinButton.setProperty("active", "False")
        self.smartCardWidget.editNameButton.style().unpolish(
            self.smartCardWidget.editNameButton
        )
        self.smartCardWidget.editPublicUrlButton.style().unpolish(
            self.smartCardWidget.editPublicUrlButton
        )
        self.smartCardWidget.editUserPinButton.style().unpolish(
            self.smartCardWidget.editUserPinButton
        )
        self.smartCardWidget.editAdminPinButton.style().unpolish(
            self.smartCardWidget.editAdminPinButton
        )
        self.smartCardWidget.editNameButton.style().polish(
            self.smartCardWidget.editNameButton
        )
        self.smartCardWidget.editPublicUrlButton.style().polish(
            self.smartCardWidget.editPublicUrlButton
        )
        self.smartCardWidget.editUserPinButton.style().polish(
            self.smartCardWidget.editUserPinButton
        )
        self.smartCardWidget.editAdminPinButton.style().polish(
            self.smartCardWidget.editAdminPinButton
        )
    def change_pin_on_card_slot(self, userpin, adminpin):
        "Final slot which will try to change the userpin"
        try:
            rjce.change_user_pin(adminpin.encode(
                "utf-8"), userpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog(
                "changing user pin", str(e))
            self.error_dialog.show()
            self.enable_cardcheck_thread_slot()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Changed user pin successfully."
        )
        self.success_dialog.show()
        self.enable_cardcheck_thread_slot()

    def change_admin_pin_on_card_slot(self, userpin, adminpin):
        "Final slot which will try to change the adminpin"
        try:
            rjce.change_admin_pin(adminpin.encode(
                "utf-8"), userpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog(
                "changing admin pin", str(e)
            )
            self.error_dialog.show()
            self.enable_cardcheck_thread_slot()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Changed admin pin successfully."
        )
        self.success_dialog.show()
        self.enable_cardcheck_thread_slot()

    def set_url_on_card_slot(self, publicURL, adminpin):
        "Final slot which will try to change the publicURL"
        try:
            rjce.set_url(publicURL.encode("utf-8"), adminpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog(
                "adding public URL", str(e))
            self.error_dialog.show()
            self.enable_cardcheck_thread_slot()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Added public URL successfully."
        )
        self.success_dialog.show()
        self.enable_cardcheck_thread_slot()

    def set_name_on_card_slot(self, name, adminpin):
        "Final slot which will try to change the name"
        try:
            # If input is "First Middle Last",
            # the parameter sent should be "Last<<Middle<<First"
            name = "<<".join(name.split()[::-1])
            rjce.set_name(name.encode("utf-8"), adminpin.encode("utf-8"))
        except Exception as e:
            self.error_dialog = MessageDialogs.error_dialog(
                "adding name", str(e))
            self.error_dialog.show()
            self.enable_cardcheck_thread_slot()
            return
        self.success_dialog = MessageDialogs.success_dialog(
            "Added name successfully.")
        self.success_dialog.show()
        self.enable_cardcheck_thread_slot()

    def disable_generate_button(self):
        self.cardcheck_thread.flag = False
        self.generateButton.setEnabled(False)
        self.update()
        self.repaint()

    def enable_generate_button(self):
        self.enable_cardcheck_thread_slot()
        self.setEnabled(True)
        self.generateButton.setEnabled(True)
        self.update()
        self.repaint()

    def show_generate_dialog(self):
        "Shows the form to generate new key"
        self.disable_cardcheck_thread_slot()
        new_key_form = NewKeyFormWidget(self.ks, self.restore_list_view)
        self.setCentralWidget(new_key_form)

    def restore_list_view(self):
        self.enable_cardcheck_thread_slot()
        self.tabs = QtWidgets.QTabWidget()
        self.keyWidget = KeyViewWidget(self.ks, self)
        self.tabs.addTab(self.keyWidget, "Available Keys")
        self.smartCardWidget = SmartCardViewWidget(self)
        self.tabs.addTab(self.smartCardWidget, "Smart Card Settings")
        self.setCentralWidget(self.tabs)

    def enable_mainwindow(self):
        self.enable_cardcheck_thread_slot()
        self.setEnabled(True)

    def disable_cardcheck_thread_slot(self):
        self.cardcheck_thread.flag = False
        self.cardcheck_thread.quit()

    def enable_cardcheck_thread_slot(self):
        self.cardcheck_thread.flag = True
        self.cardcheck_thread.start()

    def upload_to_smartcard(self):
        "Shows the userinput dialog to upload the selected key to the smartcard"
        # This means no key is selected on the list
        if not self.keyWidget.widget.selectedItems():
            self.error_dialog = MessageDialogs.error_dialog(
                "upload to smart card", "Please select a key from the list."
            )
            self.error_dialog.show()
            return

        self.disable_cardcheck_thread_slot()
        self.setEnabled(False)
        item = self.keyWidget.widget.selectedItems()[0]
        kw = self.keyWidget.widget.itemWidget(item)
        self.current_key = kw.key
        self.sccd = SmartCardConfirmationDialog(
            self.get_pins_and_passphrase_and_write,
            key=kw.key,
            enable_window=self.enable_mainwindow,
        )
        self.sccd.show()

    def get_pins_and_passphrase_and_write(
        self, passphrase: str, adminpin: str, whichkeys: int
    ):
        "This method uploads the cert to the card"
        certdata = self.current_key.keyvalue
        try:
            rjce.upload_to_smartcard(
                certdata, adminpin.encode("utf-8"), passphrase, whichkeys
            )
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
        self.setEnabled(True)
        self.enable_cardcheck_thread_slot()

    def export_public_key(self):
        # This means no key is selected on the list
        if not self.keyWidget.widget.selectedItems():
            self.error_dialog = MessageDialogs.error_dialog(
                "exporting public key", "Please select a key from the list."
            )
            self.error_dialog.show()
            return

        item = self.keyWidget.widget.selectedItems()[0]
        kw = self.keyWidget.widget.itemWidget(item)
        if key_utils.export_public_key(self, kw.key.fingerprint, kw.key.get_pub_key()):
            self.success_dialog = MessageDialogs.success_dialog(
                "Exported public key successfully!"
            )
            self.success_dialog.show()

    def exit_process(self):
        self.cardcheck_thread.flag = False
        time.sleep(1)
        sys.exit(0)

    def closeEvent(self, event):
        self.cardcheck_thread.flag = False
        time.sleep(1)
        return super().closeEvent(event)


def main():
    os.environ["QT_MAC_WANTS_LAYER"] = "1"
    app = QtWidgets.QApplication(sys.argv)
    form = MainWindow()
    form.show()
    app.exec_()


if __name__ == "__main__":
    main()
