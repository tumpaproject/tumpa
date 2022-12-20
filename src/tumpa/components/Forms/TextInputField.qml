import QtQuick
import QtQuick.Controls

Item {
    property string labelString: ""
    property string type: "text"

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

    TextField {
        id: input
        font.pixelSize: 14
        echoMode: parent.type == "password" ? TextInput.Password : TextInput.Normal

        height: 30
        width: parent.width
        anchors {
            left: parent.left
            top: label.bottom
            topMargin: 14
        }
        leftPadding: 10
        rightPadding: 10
        verticalAlignment: TextInput.AlignVCenter

        background: Rectangle {
            color: "transparent"
            border.color: "#C4C4C4"
            border.width: 1
            radius: 5
        }

        Accessible.name: parent.labelString
        Accessible.passwordEdit: parent.type == "password"
    }
}
