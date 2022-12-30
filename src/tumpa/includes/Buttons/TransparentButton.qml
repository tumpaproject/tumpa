import QtQuick
import QtQuick.Controls

Button {
    required property string labelString
    property string iconSrc: ""

    id: control
    text: labelString
    font.weight: 500
    font.pixelSize: 12

    contentItem: Text {
        text: control.text
        font: control.font
        opacity: enabled ? 1.0 : 0.3
        color: "#3730A3"
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
        anchors.leftMargin: iconSrc ? 20 : 0
        anchors.left: parent.left
    }

    background: Rectangle {
        implicitWidth: implicitContentWidth + (iconSrc ? 20 : 0)
        implicitHeight: 16
        opacity: enabled ? 1 : 0.3
        color: "transparent"
        border.color: control.visualFocus ? "#54298B" : "transparent"
        border.width: control.visualFocus ? 2 : 0
        Image {
            id: icon
            source: iconSrc
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            width: 16
            height: 16
            fillMode: Image.PreserveAspectFit
            sourceSize.width: 1024
            sourceSize.height: 1024
            mipmap: true
        }
    }
}
