import QtQuick
import QtQuick.Controls
import Qt.labs.platform

import "includes/Forms"
import "includes/Buttons"
import "includes/Utils"

Rectangle {
    id: root
    color: "white"
    property alias emails: emailTxt.text
    property alias name: nameTxt.text
    property alias passphrase: passphraseTxt.text
    property alias expirationDate: expirationDateTxt.text
    property alias keyAlgo: keyAlgorithmTxt.dropdownValue
    property alias encryptionChecked: keyTypeCheckbox.encryptionChecked
    property alias signingChecked: keyTypeCheckbox.signingChecked
    property alias authenticationChecked: keyTypeCheckbox.authenticationChecked

    signal next
    signal back

    ListModel {
        id: keyAlgoOptions
        ListElement {
            text: "Curve 25519"
            value: "curve25519"
        }
        ListElement {
            text: "RSA 4096"
            value: "rsa4096"
        }
    }

    ScrollView {
        anchors.fill: parent
        contentWidth: centerColumn.width
        anchors.leftMargin: 14

        Column {
            id: centerColumn
            spacing: 10

            Text {
                id: pathTxt
                text: qsTr("Generate new key")
                topPadding: 24
                bottomPadding: 10
                font.pixelSize: 20
                font.weight: 700
            }

            TextInputField {
                id: nameTxt
                width: 645
                labelString: qsTr("Your Name:")
                // This helps to set the current height of the box
            }

            ColSpacer {
                height: 8
            }

            TextareaInputField {
                id: emailTxt
                width: 645
                labelString: qsTr("Email addresses:")
            }

            PasswordInputField {
                id: passphraseTxt
                width: 645
                labelString: qsTr("Key Passphrase: ")
                instruction: qsTr("Recommended: 10+ chars in length")
            }

            ColSpacer {
                height: 8
            }

            TextInputField {
                id: expirationDateTxt
                width: 645
                labelString: qsTr("Expiration date (YYYY/MM/DD):")
                inputMask: "0000/00/00;_"
                // This helps to set the current height of the box
            }

            ColSpacer {
                height: 8
            }

            KeyTypeCheckboxGroup {
                id: keyTypeCheckbox
                labelString: "Key type: "
            }

            DropdownInputField {
                id: keyAlgorithmTxt
                labelString: "Key Algorithm:"
                dropdownItems: keyAlgoOptions
            }

            ColSpacer {
                height: 20
            }

            Rectangle {
                width: parent.width
                height: 70

                DefaultButton {
                    id: backKeyBttn
                    anchors {
                        bottom: parent.bottom
                        left: parent.left
                        bottomMargin: 24
                    }

                    labelString: "Back"
                    iconSrc: "../../images/backIcon.svg"

                    onClicked: {
                        root.back()
                    }
                }

                PrimaryButton {
                    id: saveKeyBttn
                    anchors {
                        bottom: parent.bottom
                        right: parent.right
                        bottomMargin: 24
                    }
                    labelString: qsTr("Create")
                    iconSrc: "../../images/tick_mark.svg"

                    onClicked: {
                        root.next()
                    }
                }
            }
        }
    }
}
