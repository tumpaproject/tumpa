import QtQuick
import QtQuick.Window
import QtQuick.Controls

import "components/Forms"
import "components/Buttons"

ApplicationWindow {
    title: qsTr("Tumpa")
    width: 900
    height: 551
    visible: true

    id: root

    // This defines if we will allow saving private key
    property bool allowsecret: false

    SplitView {
        anchors.fill: parent

        // Tags of the left side
        Rectangle {
            color: "#54298B"
            height: root.height
            SplitView.minimumWidth: 217

            Column {
                anchors {
                    topMargin: 13
                    top: parent.top
                    leftMargin: 18
                    left: parent.left
                }
                Image {
                    id: tumpaLogo
                    source: "images/logo.png"
                }
                Row {
                    topPadding: 45
                    spacing: 11

                    Image {
                        id: keyIcon
                        source: "images/key_icon.svg"
                        anchors.verticalCenter: keymgmtTxt.verticalCenter
                    }

                    Text {
                        id: keymgmtTxt
                        text: "Key Management"
                        color: "white"
                    }
                }
            }
        }

        Rectangle {
            id: bigBox
            color: "white"
            height: root.height
            SplitView.minimumWidth: 683


            ListModel {
                id: keyTypeOptions
                ListElement { name: "Encryption subkey"; value: "encryption_subkey"; isChecked: true }
                ListElement { name: "Signing subkey"; value: "signing_subkey"; isChecked: true }
                ListElement { name: "Authentication subkey"; value: "authentication_subkey"; isChecked: false }
            }

            ListModel {
                id: keyAlgoOptions
                ListElement { text: "RSA"; value: "rsa" }
                ListElement { text: "Curve 25519"; value: "curve25519" }
            }

            ScrollView {
                anchors.fill: parent
                contentWidth: column.width

                Column {
                    id: column
                    anchors.fill: parent
                    anchors.margins: 15
                    spacing: 30

                    TextInputField {
                        labelString: "Your Name:"
                    }

                    TextareaInputField {
                        labelString: "Your Email(s):"
                    }

                    PasswordInputField {
                        labelString: "Your password:"
                        instruction: "Password should be atleast 8 characters."
                    }

                    CheckboxInputField {
                        labelString: "Key type: "
                        checkboxItems: keyTypeOptions
                    }

                    DropdownInputField {
                        labelString: "Key Algorithm:"
                        dropdownItems: keyAlgoOptions
                    }

                    PrimaryButton {
                        labelString: "Confirm"
                        iconSrc: "../../images/tick_mark.svg"
                    }

                    SecondaryButton {
                        labelString: "Export public key"
                        iconSrc: "../../images/tick_mark.svg"
                        isThin: true
                    }

                    Label {
                        text: "Buffer element"
                    }
                }

            }
        }
    }
}
