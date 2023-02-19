import QtQuick

import "includes/Forms"
import "includes/Buttons"

Rectangle {
    id: root
    color: "white"
    signal genkeyclicked

    Text {
        id: nokeyTxt
        text: qsTr("Wait for the operation to finish.")
        font.pixelSize: 20
        font.weight: 700
        anchors.top: root.top
        anchors.topMargin: 200
        anchors.left: root.left
        anchors.leftMargin: 183
        color: "black"
    }
}
