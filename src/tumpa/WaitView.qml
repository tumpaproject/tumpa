import QtQuick

import "includes/Forms"
import "includes/Buttons"

Rectangle {
    id: root
    color: "white"
    signal genkeyclicked

    Item {
        id: loadingAnim
        width: 100
        height: 100
        anchors.top: root.top
        anchors.topMargin: root.height / 2 - nokeyTxt.height - 120
        anchors.horizontalCenter: parent.horizontalCenter
        clip:true

        Rectangle{
            id: circ
            width: parent.width
            height: parent.height
            border.width: 5
            radius:1000
            border.color: "#54298B"
        }

        Rectangle{
            id:mask
            width: parent.width
            height: parent.height/4
            anchors.bottom: parent.bottom
            z:4
        }

        RotationAnimator on rotation {
            from: 0
            to: 360
            duration: 2000
            loops: Animation.Infinite
        }
    }

    Text {
        id: nokeyTxt
        text: qsTr("Please wait for the operation to finish.")
        font.pixelSize: 18
        font.weight: 600
        anchors.top: loadingAnim.bottom
        anchors.topMargin: 36
        anchors.horizontalCenter: parent.horizontalCenter
        color: "black"
    }
}
