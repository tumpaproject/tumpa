import QtQuick
import QtQuick.Controls

CheckBox {

    id: control
    height: 17
    anchors.topMargin: 14
    opacity: enabled ? 1 : 0.6

    indicator: Rectangle {
        id: indicatorRect
        implicitWidth: 16
        implicitHeight: 16
        x: control.leftPadding
        y: parent.height / 2 - height / 2
        radius: 2
        border.width: control.visualFocus ? 2 : 1
        border.color: control.visualFocus ? "#54298B" : "#C4C4C4"


        Text {
            id: tickMark
            text: '\u2713' // CHECK MARK
            anchors.centerIn: indicatorRect
            visible: control.checked
            font.pixelSize: 12
        }
    }

    contentItem: Text {
        id: contentText
        text: control.text
        font.pixelSize: 14
        verticalAlignment: Text.AlignVCenter
        leftPadding: control.indicator.width + 8
    }
}