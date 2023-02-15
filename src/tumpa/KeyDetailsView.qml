import QtQuick
import QtQuick.Controls
import Qt.labs.platform

import "includes/Buttons"
import "includes/Utils"

Rectangle {
    property var keyDetailsList: null

    id: root
    color: "white"

    signal next

    Column {
        id: centerColumn
        anchors.top: root.top
        anchors.left: root.left
        anchors.right: root.right
        anchors.leftMargin: 14
        spacing: 10

        Text {
            id: pathTxt
            text: qsTr("Smartcard Details")
            topPadding: 24
            bottomPadding: 10
            font.pixelSize: 20
            font.weight: 700
        }

        ListView {
            width: root.width
            height: keyDetailsList.count * 30
            spacing: 10
            model: keyDetailsList
            delegate: KeyValRow {
                keyName: name
                valueName: value
            }
        }

//        KeyValRow {
//            keyName: qsTr("Reader:")
//            valueName: qsTr("Yubico")
//        }
//        KeyValRow {
//            keyName: qsTr("Application ID:")
//            valueName: qsTr("9923759847269HKJ")
//        }
//        KeyValRow {
//            keyName: qsTr("Application Type:")
//            valueName: qsTr("OpenPGP")
//        }
//        KeyValRow {
//            keyName: qsTr("Version:")
//            valueName: qsTr("2.1")
//        }
//        KeyValRow {
//            keyName: qsTr("Name of Cardholder:")
//            valueName: qsTr("Saptaks")
//        }
//        KeyValRow {
//            keyName: qsTr("Public URL of Cardholder:")
//            valueName: qsTr("OpenPGP")
//        }
    }

    ModalDangerButton {
        id: formatKeyBtn
        anchors {
            bottom: root.bottom
            right: root.right
            rightMargin: 24
            bottomMargin: 24
        }

        labelString: "Format smartcard"

        onClicked: {
            console.log("Format smartcard button clicked")
        }
    }
}
