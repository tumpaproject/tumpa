import QtQuick
import QtQuick.Window
import QtQuick.Controls

import "includes/Forms"
import "includes/Buttons"
import "includes/Utils"

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

            Image {
                id: tumpaLogo
                source: "images/logo.svg"
                anchors {
                    topMargin: 14
                    top: parent.top
                    leftMargin: 14
                    left: parent.left
                }
            }

            Column {
                id: menuOptions
                anchors {
                    topMargin: 20
                    top: tumpaLogo.bottom
                    leftMargin: 14
                    left: parent.left
                    right: parent.right
                    rightMargin: 10
                }
                spacing: 4

                LeftIconButton {
                    id: leftIconBttn
                    // This is our initial active menu item
                    active: true
                    anchors.bottomMargin: 18
                    text: qsTr("Key Management")
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("leftIconBttn")
                    }
                }

                ColSpacer {
                    height: 10
                }

                LeftIconButton {
                    id: leftKeyBttn
                    // This is our Yubikey button
                    imageSource: "../../images/usbkey.svg"
                    text: qsTr("Smart Card")
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("leftKeyBttn")
                    }
                }

                LeftMenuButton {
                    id: editNameBttn
                    text: qsTr("Edit Name")
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("editNameBttn")
                    }
                }

                LeftMenuButton {
                    id: editPublicURLBttn
                    text: qsTr("Edit Public URL")
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("editPublicURLBttn")
                    }
                }

                LeftMenuButton {
                    id: editUserPinBttn
                    text: qsTr("Change User Pin")
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("editUserPinBttn")
                    }
                }

                LeftMenuButton {
                    id: editAdminPinBttn
                    text: qsTr("Change Admin Pin")
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("editAdminPinBttn")
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

                    KeyTypeCheckboxGroup {
                        labelString: "Key type: "
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
                        labelString: "Export public key (press to see info modal)"
                        iconSrc: "../../images/export.svg"
                        isThin: true
                        onClicked: {
                            var component1 = Qt.createComponent("includes/Utils/InfoModal.qml");
                            var win = component1.createObject(
                                root,
                                {
                                    headingText: "Expired Key",
                                    contentText: "This key is expired and cannot be send to your card."
                                }
                            );
                            win.okayed.connect(()=>{
                                  console.log("Modal okayed");
                                  win.destroy();
                              });
                            win.show();
                        }
                    }

                    DangerButton {
                        labelString: "Revoke key (Press to see danger modal)"
                        iconSrc: "../../images/revoke.svg"
                        isThin: true
                        onClicked: {
                            var component1 = Qt.createComponent("includes/Utils/WarningModal.qml");
                            var win = component1.createObject(
                                root,
                                {
                                    dangerBtnText: "Revoke",
                                    headingText: "Warning",
                                    contentText: "Are you sure you want to revoke the email@test.com user ID? This action can’t be undone."
                                }
                            );
                            win.accepted.connect(()=>{
                                  console.log("Modal accepted");
                                  win.destroy();
                              });
                            win.rejected.connect(()=>{
                                  console.log("Modal rejected");
                                  win.destroy();
                              });
                            win.show();
                        }
                    }

                    DefaultButton {
                        labelString: "Back"
                        iconSrc: "../../images/backIcon.svg"
                    }

                    ModalDangerButton {
                        labelString: "Revoke"
                    }

                    TransparentButton {
                        labelString: "Export Public Key"
                        iconSrc: "../../images/export.svg"
                    }

                    Label {
                        text: "Buffer element"
                    }
                }

            }
        }
    }
    function clearAcitve() {
        for (var i in menuOptions.children) {
            if (!(menuOptions.children[i] instanceof ColSpacer)) {
                menuOptions.children[i].active = false
            }
        }
    }
}
