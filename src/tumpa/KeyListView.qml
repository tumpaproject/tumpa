import QtQuick
import QtQuick.Controls

import "includes/Buttons"
import "includes/Utils"

Rectangle {
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
            text: qsTr("All keys")
            topPadding: 14
            bottomPadding: 14
            font.pixelSize: 20
            font.weight: 700
        }

        KeyItem {
            width: 645

            fingerprint: qsTr("49CC5563EEE747C8F6C801037D0E7EF2AEDC5E84")
            createdOn: qsTr("10 March 2022")
            expiresOn: qsTr("10 March 2023")
        }

        KeyItem {
            width: 645

            fingerprint: qsTr("49CC5563EEE747C8F6C801037D0E7EF2AEDC5B10")
            createdOn: qsTr("19 March 2022")
            expiresOn: qsTr("10 March 2025")
        }
    }
}
