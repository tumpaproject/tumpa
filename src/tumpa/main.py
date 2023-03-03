# This Python file uses the following encoding: utf-8
import sys
import os.path
from pathlib import Path
from typing import List, Optional, final
from pathlib import Path
import datetime
import json


# Hack for running in Qt Creator
if __name__ == "__main__":
    parent_path = Path(__file__)
    sys.path.insert(0, str(parent_path.parent.parent))

from PySide6.QtGui import QGuiApplication, QFontDatabase, QFont
from PySide6.QtQml import QQmlApplicationEngine
from PySide6.QtCore import QThread, Signal, Slot, QObject, Property

from PySide6 import QtGui as qtg
from PySide6 import QtCore as qtc
from PySide6 import QtQml as qml

from johnnycanencrypt import Cipher, Key
import johnnycanencrypt.johnnycanencrypt as rjce
import johnnycanencrypt as jce

from tumpa.configuration import get_keystore_directory


class KeyThread(QThread):
    "Generates a new key"
    updated = Signal()

    def __init__(self, ks: jce.KeyStore):
        QThread.__init__(self)
        self.ks = ks
        self.uids = []
        self.password = ""
        self.whichkeys = 0
        self.expiration = None
        self.keytype: Optional[Cipher] = None
        self.key: Optional[jce.Key] = None

    def setup(
        self,
        uids,
        password,
        whichkeys,
        keytype: Cipher = Cipher.Cv25519,
        expiration=None,
        canexpire=True,
    ):
        "To clean itself."
        self.uids = uids
        self.password = password
        self.whichkeys = whichkeys
        self.expiration = expiration  # Timestamp
        self.keytype = keytype
        self.canexpire = canexpire

    # run method gets called when we start the thread
    # This is where we will generate the OpenPGP key
    def run(self):
        if isinstance(self.keytype, Cipher):
            self.key = self.ks.create_key(
                self.password,
                self.uids,
                self.keytype,
                None,
                self.expiration,
                self.canexpire,
                self.whichkeys,
                can_primary_sign=False,
                can_primary_expire=self.canexpire,
            )
            self.updated.emit()


def convert_date_to_text(d: datetime = None) -> str:
    "Converts the datetime for UI"
    if not d:
        return ""
    return d.strftime("%d %B %Y")


def convert_none_to_text(data) -> str:
    "To convert possible None values"
    if not data:
        return ""
    return data


class KeyList:
    def __init__(self, data) -> None:
        self.keys: List[jce.Key] = data

    def json(self) -> str:
        "returns a JSON string to QML"
        result = []
        for entry in self.keys:
            data = {}
            data["fingerprint"] = entry.fingerprint
            data["creationtime"] = convert_date_to_text(entry.creationtime)
            data["expirationtime"] = convert_date_to_text(entry.expirationtime)
            data["uids"] = entry.uids
            data["keyid"] = entry.keyid
            data["keytype"] = entry.keytype.value
            data["can_primary_sign"] = entry.can_primary_sign
            data["primary_on_card"] = convert_none_to_text(entry.primary_on_card)
            data["oncard"] = convert_none_to_text(entry.oncard)
            subkeys = []
            for subkey in entry.othervalues["subkeys_sorted"]:
                subkey["creation"] = convert_date_to_text(subkey["creation"])
                subkey["expiration"] = convert_date_to_text(subkey["expiration"])
                subkeys.append(subkey)

            # Now add it to the our data
            data["subkeys"] = subkeys
            result.append(data)

        return json.dumps(result)


class TBackend(QObject):
    "Main backend class for all operations"
    updated = Signal()
    uploaded = Signal()
    errored = Signal()
    refreshKeys = Signal()

    def __init__(self, ks: Optional[jce.KeyStore] = None):
        super(TBackend, self).__init__(None)
        self.havekeys = False
        if ks:
            self.ks = ks
        else:
            self.ks = jce.KeyStore(get_keystore_directory())
        # Create the keystore if not there
        self.ks.upgrade_if_required()
        # This data we will pass to QML
        self.keylist = KeyList(self.ks.get_all_keys())

        self.kt = KeyThread(self.ks)
        self.kt.updated.connect(self.key_generation_done)

    @Slot(result=str)
    def get_keys_json(self) -> str:
        "To get the JSON from inside"
        data = self.keylist.json()
        return data

    def get_havekeys(self):
        return self.havekeys

    @Slot(str, str, str, str, bool, bool, bool, str)
    def generateKey(
        self,
        name,
        qemails,
        password,
        expiry_date: str,
        encryption,
        signing,
        authentication,
        keytype,
    ) -> None:
        "Setup all the details and then try to generate a new key"
        emails = [email.strip() for email in qemails.split("\n")]
        uids = [f"{name} <{email}>" for email in emails]
        password = password.strip()
        # By default we assume keys are expiring
        canexpire = False
        expiry = None

        expiry_date = expiry_date.strip()
        if not expiry_date.startswith("/"):
            # TODO: validate the expiry_date
            # For now I am assuming it is correct
            dates = expiry_date.split("/")
            year = int(dates[0])
            month = int(dates[1])
            day = int(dates[2])
            # we have a proper expiry date
            expiry = datetime.datetime(year, month, day)
            canexpire = True

        # Figure out what kind of key to generate
        if keytype == "rsa4096":
            key_algo = Cipher.RSA4k
        else:
            key_algo = Cipher.Cv25519

        whichsubkeys = 0
        if encryption:
            whichsubkeys += 1
        if signing:
            whichsubkeys += 2
        if authentication:
            whichsubkeys += 4

        # Now feed in the data to the other thread
        self.kt.setup(uids, password, whichsubkeys, key_algo, expiry, canexpire)
        # Start the thread
        self.kt.start()

    @Slot()
    def key_generation_done(self):
        "Receives information that key generation is done"
        allkeys = self.ks.get_all_keys()
        allkeys.sort(key=lambda x: get_creationtime(x))
        self.havekeys = True
        print(f"{self.havekeys=}")
        # TODO: update the datamodel.
        self.updated.emit()

    @Slot(str, str, result=bool)
    def updateName(self, name, adminpin):
        "Updates the name in the Yubikey"
        name = name.strip()
        words = name.split()
        if len(words) > 1:
            finalname = f"{words[1]}<<{words[0]}"
        else:
            finalname = f"{words[0]}<<"
        try:
            return rjce.set_name(finalname.encode("utf-8"), adminpin.encode("utf-8"))
        except:
            # TODO: Add debug log here
            return False

    @Slot(str, str, result=bool)
    def updateURL(self, url, adminpin):
        "Updates the url in the Yubikey"
        url = url.strip()
        try:
            return rjce.set_url(url.encode("utf-8"), adminpin.encode("utf-8"))
        except:
            # TODO: Add debug log here
            return False

    @Slot(str, str, result=bool)
    def updateUserPin(self, pin, adminpin):
        "Updates the user pin in the Yubikey"
        pin = pin.strip()
        try:
            return rjce.change_user_pin(adminpin.encode("utf-8"), pin.encode("utf-8"))
        except:
            # TODO: Add debug log here
            return False

    @Slot(str, str, result=bool)
    def updateAdminPin(self, newpin, adminpin):
        "Updates the admin pin in the Yubikey"
        newpin = newpin.strip()
        try:
            return rjce.change_admin_pin(
                adminpin.encode("utf-8"), newpin.encode("utf-8")
            )
        except:
            # TODO: Add debug log here
            return False

    haveKeys = Property(bool, get_havekeys, None)


def get_creationtime(x: jce.Key):
    if x.creationtime:
        return x.creationtime
    return 0


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

    # Adding the backend into the story

    p = TBackend()
    ctx = engine.rootContext()
    ctx.setContextProperty("tbackend", p)
    qml_file = Path(__file__).resolve().parent / "main.qml"
    engine.load(qml_file)
    if not engine.rootObjects():
        sys.exit(-1)
    # Let us update the keys list
    p.refreshKeys.emit()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
