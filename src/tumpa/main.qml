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
        id: keyDetails

        ListElement{
            name: "Application Type: "
            value: "OpenPGP"
        }

        ListElement{
            name: "Name of Cardholder: "
            value: "SaptakS"
        }

        ListElement{
            name: "Public URL of Cardholder: "
            value: "https://saptaks.website"
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

            KeyDetailsView {
                keyDetailsList: keyDetails
                anchors.fill: parent
            }
            //            StartView {

            //                anchors.fill: parent
            //            }
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
