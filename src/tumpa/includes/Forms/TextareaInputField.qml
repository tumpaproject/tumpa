import QtQuick
import QtQuick.Controls

Item {
    id: root
    property string labelString: ""
    property alias text: input.text

    anchors.leftMargin: 20
    height: 130
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

    ScrollView {
        height: 99
        width: parent.width
        anchors {
            left: parent.left
            top: label.bottom
            topMargin: 14
        }

        TextArea {
            id: input
            font.pixelSize: 14
            leftPadding: 10
            rightPadding: 10
            topPadding: 6
            bottomPadding: 6

            background: Rectangle {
                color: "transparent"
                border.color: "#C4C4C4"
                border.width: 1
                radius: 5
            }

            KeyNavigation.priority: KeyNavigation.BeforeItem
            KeyNavigation.tab: nextItemInFocusChain()

            Accessible.name: root.labelString
        }
    }
}
