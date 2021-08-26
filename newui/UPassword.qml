import QtQuick 2.0

Column{
    spacing: 5
    property alias inputtext: inputTxt.text
    property alias inputpinvalue: pinInput.text
    property alias pinbox: pinInput

    Text {
        id: inputTxt
        text: "Current Admin Pin"
    }
    Rectangle {
        width: 300
        height: 40
        border.color: "gainsboro"
        border.width: 1


        TextInput {
            id: pinInput
            anchors.fill: parent
            width: 200
            height: 60
            font.pointSize: 20
            echoMode: TextInput.Password
            clip: true

        }
    }
}
