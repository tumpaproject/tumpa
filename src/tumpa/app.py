# This Python file uses the following encoding: utf-8
from pprint import pprint
import sys
import os.path
import copy
from pathlib import Path
from typing import List, Optional, Tuple
from pathlib import Path
import datetime
import json


# Hack for running in Qt Creator
if __name__ == "__main__":
    parent_path = Path(__file__)
    sys.path.insert(0, str(parent_path.parent.parent))

from PySide6.QtGui import QGuiApplication, QFontDatabase, QFont
from PySide6.QtQml import QQmlApplicationEngine, qmlRegisterType
from PySide6.QtCore import QThread, Signal, Slot, QObject, Property

from PySide6 import QtGui as qtg
from PySide6 import QtCore as qtc
from PySide6 import QtQml as qml

from johnnycanencrypt import Cipher, Key
import johnnycanencrypt.johnnycanencrypt as rjce
import johnnycanencrypt as jce

from tumpa.configuration import get_keystore_directory


class SubkeyType(QObject):
    def __init__(self, sign, enc, auth) -> None:
        QObject.__init__(self)
        self.s = sign
        self.e = enc
        self.a = auth
        self.fp = ""

    def read_s(self):
        return self.s

    def read_e(self):
        return self.e

    def read_a(self):
        return self.a
    def read_fingerprint(self):
        return self.fp

    sign = Property(bool, read_s, None, constant=True)
    encryption = Property(bool, read_e, None, constant=True)
    authentication = Property(bool, read_a, None, constant=True)
    fingerprint = Property(str, read_fingerprint, None, constant=True)


# TODO: Fix this stupid hack
subkeytypes = SubkeyType(False, False, False)


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

    def __len__(self) -> int:
        return len(self.keys)

    def json(self) -> str:
        "returns a JSON string to QML"
        result = []
        for entry_value in self.keys:
            # To make sure that we have fresh data everytime
            entry = copy.deepcopy(entry_value)
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
        self.fp = ""
        self.havekeys = False
        if ks:
            self.ks = ks
        else:
            self.ks = jce.KeyStore(get_keystore_directory())
        # Create the keystore if not there
        self.ks.upgrade_if_required()
        self.kt = KeyThread(self.ks)
        self.kt.updated.connect(self.key_generation_done)
        self.safe_update_keylist()

    def safe_update_keylist(self):
        # This data we will pass to QML
        try:
            self.keylist = KeyList(self.ks.get_all_keys())
        except jce.exceptions.KeyNotFoundError:
            self.keylist = KeyList([])


    @Slot(result=str)
    def get_keys_json(self) -> str:
        "To get the JSON from inside"
        data = self.keylist.json()
        return data

    @Slot(result=str)
    def get_card_json(self) -> str:
        "Get the card details as JSON"
        try:
            data = rjce.get_card_details()
        except:
            # For any error
            data = {}
        if "name" in data:
            name = data["name"]
            words = name.split("<<")
            if len(words) > 1:
                finalname = f"{words[1]} {words[0]}"
            else:
                finalname = f"{words[0]}"
        else:
            finalname = ""

        results = []
        # Now let us create list of items
        results.append(
            {"name": "Serial Number", "value": data.get("serial_number", "000")}
        )
        results.append({"name": "Name", "value": finalname})
        results.append({"name": "Public URL", "value": data.get("url", "")})
        results.append(
            {"name": "User Pin retires left", "value": str(data.get("PW1", 0))}
        )
        results.append(
            {"name": "Reset Pin retries left", "value": str(data.get("RC", 0))}
        )
        results.append(
            {"name": "Admin Pin retires left", "value": str(data.get("PW3", 0))}
        )
        results.append(
            {"name": "Signatures made", "value": str(data.get("signatures", 0))}
        )

        return json.dumps(results)

    def get_havekeys(self):
        result = len(self.keylist) > 0
        return result

    def get_havecard(self):
        "To verify if we have a card connected"
        return rjce.is_smartcard_connected()

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
        print(f"{whichsubkeys=}")
        # Now feed in the data to the other thread
        self.kt.setup(uids, password, whichsubkeys, key_algo, expiry, canexpire)
        # Start the thread
        self.kt.start()

    @Slot()
    def key_generation_done(self):
        "Receives information that key generation is done"
        self.keylist = KeyList(self.ks.get_all_keys())
        self.havekeys = True
        print(f"{self.havekeys=}")
        # TODO: update the datamodel.
        self.updated.emit()

    @Slot(str)
    def removeKey(self, fingerprint):
        "Removes the key from the store"
        self.ks.delete_key(fingerprint)
        # Now get the new list of keys
        self.safe_update_keylist()

    @Slot(str)
    def get_subkey_types(self, fingerprint: str):
        key = self.ks.get_key(fingerprint)
        data = available_subkeys(key)
        print(f"{data=}")
        e, s, a =  data
        # TODO: the stupid hack to pass data to QML
        subkeytypes.e = e
        subkeytypes.s = s
        subkeytypes.a = a
        subkeytypes.fp = fingerprint

    @Slot(str)
    def current_fingerprint(self, fingerprint: str):
        "Sets the current fingerprint for any future work."
        self.fp = fingerprint

    @Slot(str, result=str)
    def save_public_key(self, filepath: str):
        "Saves the current key as publick key"
        try:
            key = self.ks.get_key(self.fp)
        except:
            # FIXME: handle error here
            return f"Error while finding the key for {self.fp}"
        if filepath.startswith("file://"):
            filepath = filepath[7:]
        public_key = key.get_pub_key()
        try:
            with open(filepath, "w") as fobj:
                fobj.write(public_key)
        except:
            return f"Error while saving the file at {filepath}."
        return "success"

    @Slot(str, str, bool, int, result=str)
    def uploadKey(self, fingerprint: str, password: str, only_subkeys: bool, whichsubkeys: int):
        print(f"Received {fingerprint=}  and {password=} {only_subkeys=} {whichsubkeys=}")
        # First get the key
        key = self.ks.get_key(fingerprint)
        # reset the yubikey
        result = rjce.reset_yubikey()
        if not result:
            return "Failed to reset Yubikey"
        try:
            rjce.upload_to_smartcard(key.keyvalue, b"12345678", password, whichsubkeys)
        except:
            return "Failed to upload to the Yubikey."
        return "success"

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

    haveKeys = Property(bool, get_havekeys, None, constant=True)
    haveCard = Property(bool, get_havecard, None, constant=True)


def available_subkeys(key: Key) -> Tuple[bool, bool, bool]:
    "Returns bool tuple (enc, signing, auth)"
    subkeys_sorted = key.othervalues["subkeys_sorted"]
    got_enc = False
    got_sign = False
    got_auth = False
    # Loop over on the subkeys
    for subkey in subkeys_sorted:
        print(subkey)
        if subkey["revoked"]:
            continue
        if not subkey["expiration"]:
            if subkey["keytype"] == "encryption":
                got_enc = True
                continue
            if subkey["keytype"] == "signing":
                got_sign = True
                continue
            if subkey["keytype"] == "authentication":
                got_auth = True
                continue

        if (
            subkey["expiration"] is not None
            and subkey["expiration"].date() > datetime.datetime.now().date()
        ):
            if subkey["keytype"] == "encryption":
                got_enc = True
                continue
            if subkey["keytype"] == "signing":
                got_sign = True
                continue
            if subkey["keytype"] == "authentication":
                got_auth = True
                continue
    # Now return the data
    return (got_enc, got_sign, got_auth)


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
    # FIXME: Stupid hack to pass data to QML via this object
    ctx.setContextProperty("SubKeyTypes", subkeytypes)
    qml_file = Path(__file__).resolve().parent / "main.qml"
    engine.load(qml_file)
    if not engine.rootObjects():
        sys.exit(-1)
    # Let us update the keys list
    p.refreshKeys.emit()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
