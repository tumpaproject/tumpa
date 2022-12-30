import QtQuick

import "includes/Forms"
import "includes/Buttons"

Rectangle {
    id: root
    color: "white"
    signal clicked

    Image {
        id: bigKey
        source: "images/big_key.svg"
        anchors.topMargin: 140
        anchors.top: root.top
        anchors.left: root.left
        anchors.leftMargin: 326
    }

    Text {
        id: nokeyTxt
        text: qsTr("No keys added yet")
        font.pixelSize: 20
        font.weight: 700
        anchors.top: bigKey.bottom
        anchors.topMargin: 12
        anchors.left: root.left
        anchors.leftMargin: 283
        color: "black"
    }

    Text {
        id: bigMsgTxt
        text: qsTr("You can import an existing key or generate a new one")
        font.pixelSize: 14
        font.weight: 400
        anchors.top: nokeyTxt.bottom
        anchors.topMargin: 20
        anchors.left: root.left
        anchors.leftMargin: 180
        anchors.bottomMargin: 16
    }

    Row {
        id: buttonRow
        anchors.top: bigMsgTxt.bottom
        anchors.topMargin: 20
        anchors.left: root.left
        anchors.leftMargin: 178
        spacing: 10

        PrimaryButton {
            id: generateKeyBttn
            labelString: qsTr("Generate New Key")
            iconSrc: "../../images/plus.svg"

            onClicked: console.log("generate clicked")
        }

        TransparentButton {
            anchors.leftMargin: 10
            width: generateKeyBttn.width
            height: generateKeyBttn.height
            labelString: "Import Secret Key"
            iconSrc: "../../images/import.svg"

            onClicked: console.log("import clicked")
        }
    }
}
