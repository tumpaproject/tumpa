import QtQuick
import QtQuick.Controls
import Qt.labs.platform

import "includes/Forms"
import "includes/Buttons"
import "includes/Utils"

Rectangle {
    id: root
    color: "white"
    property alias password: passwordTxt.text
    property var subkeys: subkeyCheckbox
    property string fingerprint: SubKeyTypes.fingerprint

    signal next

    Column {
        id: centerColumn
        anchors.top: root.top
        anchors.left: root.left
        anchors.right: root.right
        anchors.leftMargin: 14
        spacing: 10

        Item {
            width: parent.width
            height: 24
        }

        Text {
            id: pathTxt
            text: qsTr("Upload to smartcard")
            bottomPadding: 10
            font.pixelSize: 20
            font.weight: 700
        }

        PasswordInputField {
            id: passwordTxt
            width: 645
            height: 40
            labelString: qsTr("Key password")
        }

        ColSpacer {
            height: 20
        }

        KeyTypeCheckboxGroup {
            id: subkeyCheckbox
            labelString: qsTr("Subkeys to upload")
            signingChecked: SubKeyTypes.sign
            signingEnabled: SubKeyTypes.sign
            encryptionChecked: SubKeyTypes.encryption
            encryptionEnabled: SubKeyTypes.encryption
            authenticationChecked: SubKeyTypes.authentication
            authenticationEnabled: SubKeyTypes.authentication
        }
    }

    PrimaryButton {
        id: saveKeyBttn
        anchors {
            bottom: root.bottom
            right: root.right
            rightMargin: 24
            bottomMargin: 24
        }
        labelString: qsTr("Upload")
        iconSrc: "../../images/tick_mark.svg"

        onClicked: function () {
            var win = getConfirmBox(
                        qsTr("Upload the new key?"), qsTr(
                            "This will delete current key (if any) on the Yubikey"))
            win.accepted.connect(() => {

                                     // The user agreed.
                                     // Now remove the key.
                                     win.destroy()
                                     root.next()
                                 })
            win.rejected.connect(() => {
                                     // The user canceled the operation
                                     win.destroy()
                                 })
            win.show()
        }
    }
}
