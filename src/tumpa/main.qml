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

    ListModel {
        id: keyList

        ListElement{
            fingerprint: "49CC5563EEE747C8F6C801037D0E7EF2AEDC5E84"
            createdOn: "10 March 2022"
            expiresOn: "10 March 2024"
        }

        ListElement{
            fingerprint: "49CC5563EEE747C8F6C801037D0E7EF2AEDBH213"
            createdOn: "10 March 2023"
            expiresOn: "10 March 2024"
        }

        ListElement{
            fingerprint: "501S5563EEE747C8F6C801037D0E7EF2AEDC5E84"
            createdOn: "10 March 2022"
            expiresOn: "10 February 2024"
        }

        ListElement{
            fingerprint: "501S5563EEE747C8F6C801037D0E7EF2AEDC5E84"
            createdOn: "10 March 2022"
            expiresOn: "10 February 2024"
        }

        ListElement{
            fingerprint: "501S5563EEE747C8F6C801037D0E7EF2AEDC5E84"
            createdOn: "10 March 2022"
            expiresOn: "10 February 2024"
        }
    }

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
                        clearActive()
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
                        clearActive()
                        active = true
                        console.log("leftKeyBttn")
                    }
                }

                LeftMenuButton {
                    id: editNameBttn
                    text: qsTr("Edit Name")
                    onClicked: {
                        clearActive()
                        active = true
                        console.log("editNameBttn")
                    }
                }

                LeftMenuButton {
                    id: editPublicURLBttn
                    text: qsTr("Edit Public URL")
                    onClicked: {
                        clearActive()
                        active = true
                        console.log("editPublicURLBttn")
                    }
                }

                LeftMenuButton {
                    id: editUserPinBttn
                    text: qsTr("Change User Pin")
                    onClicked: {
                        clearActive()
                        active = true
                        console.log("editUserPinBttn")
                    }
                }

                LeftMenuButton {
                    id: editAdminPinBttn
                    text: qsTr("Change Admin Pin")
                    onClicked: {
                        clearActive()
                        active = true
                        console.log("editAdminPinBttn")
                    }
                }
            }
            StatusInfo {
                anchors {
                    bottom: parent.bottom
                    bottomMargin: 24
                    left: parent.left
                    leftMargin: 14
                }

                statusText: qsTr("Card detected")
            }
        }

        Rectangle {
            id: bigBox
            color: "white"
            height: root.height
            SplitView.minimumWidth: 683

            KeyListView {
                anchors.fill: parent
                keyList: keyList
            }
            //            StartView {

            //                anchors.fill: parent
            //            }
        }
    }

    function clearActive() {
        for (var i in menuOptions.children) {
            if (!(menuOptions.children[i] instanceof ColSpacer)) {
                menuOptions.children[i].active = false
            }
        }
    }
}
