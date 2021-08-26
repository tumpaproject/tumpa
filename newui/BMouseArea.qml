import QtQuick 2.0

MouseArea {

    // To mark if the color will change while exiting from Hover
    property bool ifClicked: false
    // To find whose color needs to be changed in hover
    property var bTextID

    anchors.fill: parent
    hoverEnabled: true
    onEntered: {
        parent.border.color = "black"
        bTextID.color = "white"
        parent.color = clicked_color
    }
    onExited: {

        if (ifClicked === false) {
            parent.color = button_color
            bTextID.color = "black"
            parent.border.color = "gainsboro"
        }
    }
}
