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

            Image {
                id: tumpaLogo
                source: "images/logo.png"
                anchors {
                    topMargin: 13
                    top: parent.top
                    leftMargin: 14
                    left: parent.left
                }
            }

            Column {
                anchors {
                    topMargin: 45
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
                    onClicked: {
                        clearAcitve()
                        active = true
                        console.log("leftIconBttn")
                    }
                }

                LeftIconButton {
                    id: leftKeyBttn
                    // This is our Yubikey button
                    imageSource: "../../images/usbkey.svg"
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
        }
    }

    function clearAcitve() {
        leftIconBttn.active = false
        leftKeyBttn.active = false
        editNameBttn.active = false
        editPublicURLBttn.active = false
        editUserPinBttn.active = false
        editAdminPinBttn.active = false
    }
}
