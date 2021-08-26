import QtQuick 2.0

Column {
    spacing: 5
    property alias inputtext: nameTxt.text
    property alias inputvalue : nameInput.text
    Text {
        id: nameTxt
        text: "Your Name"
    }
    Rectangle {
        width: 300
        height: 40
        border.color: "gainsboro"
        border.width: 1


        TextInput {
            id: nameInput
            anchors.fill: parent
            anchors.margins: 10
            // anchors.bottom: parent.bottom // this does not work
            width: 200
            height: 60
            font.pointSize: 15
            clip: true

        }
    }


}
