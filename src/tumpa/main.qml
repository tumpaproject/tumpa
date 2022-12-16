import QtQuick
import QtQuick.Window
import QtQuick.Controls

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
        }
    }
}