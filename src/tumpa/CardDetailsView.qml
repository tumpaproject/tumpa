import QtQuick
import QtQuick.Controls
import Qt.labs.platform

import "includes/Buttons"
import "includes/Utils"

Rectangle {
    property var cardDetailsList: null

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
            height: cardDetailsList.count * 30
            spacing: 10
            model: cardDetailsList
            delegate: KeyValRow {
                keyName: name
                valueName: value
            }
        }
    }

    ModalDangerButton {
        id: formatKeyBtn
        anchors {
            bottom: root.bottom
            right: root.right
            rightMargin: 24
            bottomMargin: 24
        }

        labelString: qsTr("Reset smartcard")

        onClicked: {
            // TODO: reset the smartcard
            console.log("Reset smartcard button clicked")
        }
    }
}
