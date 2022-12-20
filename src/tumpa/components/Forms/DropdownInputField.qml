import QtQuick
import QtQuick.Controls

Item {
    property string labelString: ""
    property var dropdownItems: null

    anchors.leftMargin: 20
    height: 61
    width: parent.width

    Text {
        id: label
        font.pixelSize: 14
        text: parent.labelString
        anchors {
            left: parent.left
            top: parent.top
        }
    }
    ComboBox {
        id: control
        height: 30
        width: parent.width

        model:dropdownItems
        textRole: "text"
        valueRole: "value"

        anchors {
            left: parent.left
            top: label.bottom
            topMargin: 14
        }
        leftPadding: 10
        rightPadding: 10

        background: Rectangle {
            id: backgroundRect
            color: "transparent"
            border.color: control.activeFocus ? "#54298B" : "#C4C4C4"
            border.width: control.visualFocus ? 2 : 1
            radius: 5
        }

        indicator: Image {
            id: indicatorIcon
            x: control.width - width - control.rightPadding
            y: control.topPadding + (control.availableHeight - height) / 2
            source: "../../images/dropdownIndicator.svg"
            sourceSize.width: 8
            sourceSize.height: 11
        }
    }
}
