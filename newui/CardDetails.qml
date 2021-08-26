import QtQuick 2.14
import QtQuick.Layouts 1.3
import QtQuick.Controls 2.5

Item {
    id: root
    property int button_width: 140
    property int button_height: 40
    property string button_color: "ghostwhite"
    property string clicked_color: "dodgerblue"
    property int internal_window_height: 560

    // We have a row with two items
    RowLayout {
        spacing: 0
        // For the buttons
        Rectangle {
            color: "lightblue"
            width: 220
            Layout.preferredHeight: internal_window_height

            Column {
                anchors.centerIn: parent
                spacing: 60

                // Edit name
                Rectangle {
                    id: editNameID
                    width: button_width
                    height: button_height
                    color: button_color

                    // signal to bring back the button to normal
                    signal deactivate()
                    // signal to take the button to hover/clicked state
                    signal activate()

                    onDeactivate:  {
                        console.log("deactivating edit name")
                        editNameID.color = button_color
                        editNameID.border.color = "gainsboro"
                        editNameButtonID.color = "black"
                        editNameWindow.visible = false
                        mousearea_editNameID.ifClicked = false

                    }

                    onActivate: {
                        console.log("Activating edit name")
                        editNameID.color = clicked_color
                        editNameID.border.color = "black"
                        editNameButtonID.color = "white"
                        editNameWindow.visible = true
                        mousearea_editNameID.ifClicked = true
                    }

                    Text {
                        id: editNameButtonID
                        text: "Edit Name"
                        anchors.centerIn: parent
                    }
                    BMouseArea {
                        id: mousearea_editNameID
                        // Always set the bTextID
                        bTextID: editNameButtonID
                        onClicked: {
                            console.log("Edit Name button clicked")
                            parent.color = clicked_color
                            ifClicked = true
                            bTextID.color = "white"

                            // hide others
                            editURLID.deactivate()
                            editUserPinID.deactivate()
                            editAdminPinID.deactivate()

                            // Show us
                            editNameID.activate()

                        }
                    }

                }

                // Edit URL
                Rectangle {
                    id: editURLID
                    width: button_width
                    height: button_height
                    color: button_color

                    // signal to bring back the button to normal
                    signal deactivate()
                    // signal to take the button to hover/clicked state
                    signal activate()

                    onDeactivate:  {
                        console.log("deactivating edit url")
                        editURLID.color = button_color
                        editURLID.border.color = "gainsboro"
                        editURLButtonID.color = "black"
                        editPublicUrlWindow.visible = false
                        mousearea_editURLID.ifClicked = false
                    }

                    onActivate: {
                        console.log("Activating edit URL")
                        editURLID.color = clicked_color
                        editURLID.border.color = "black"
                        editURLButtonID.color = "white"
                        editPublicUrlWindow.visible = true
                    }

                    Text {
                        id: editURLButtonID
                        text: "Edit Public URL"
                        anchors.centerIn: parent
                    }
                    BMouseArea {
                        id: mousearea_editURLID
                        // Always set the bTextID
                        bTextID: editURLButtonID
                        onClicked: {
                            console.log("Edit URL button clicked")
                            parent.color = clicked_color
                            ifClicked = true
                            bTextID.color = "white"

                            // hide others
                            editNameID.deactivate()
                            editUserPinID.deactivate()
                            editAdminPinID.deactivate()

                            // show us
                            editURLID.activate()

                        }

                    }
                }

                // Edit User pin
                Rectangle {
                    id: editUserPinID
                    width: button_width
                    height: button_height
                    color: button_color

                    // signal to bring back the button to normal
                    signal deactivate()
                    // signal to take the button to hover/clicked state
                    signal activate()

                    onDeactivate:  {
                        console.log("deactivating edit user pin")
                        editUserPinID.color = button_color
                        editUserPinID.border.color = "gainsboro"
                        editUserPinButtonID.color = "black"
                        editUserPinWindow.visible = false
                        mousearea_editUserPinID.ifClicked = false
                    }

                    onActivate: {
                        console.log("Activating edit user pin")
                        editUserPinID.color = clicked_color
                        editUserPinID.border.color = "black"
                        editUserPinButtonID.color = "white"
                        editUserPinWindow.visible = true
                    }

                    Text {
                        id: editUserPinButtonID
                        text: "Edit User Pin"
                        anchors.centerIn: parent
                    }

                    BMouseArea {
                        id: mousearea_editUserPinID
                        // Always set the bTextID
                        bTextID: editUserPinButtonID
                        onClicked: {
                            console.log("Edit user pin button clicked")
                            parent.color = clicked_color
                            ifClicked = true
                            bTextID.color = "white"

                            // hide others
                            editNameID.deactivate()
                            editURLID.deactivate()
                            editAdminPinID.deactivate()

                            // show us
                            editUserPinID.activate()

                        }

                    }


                }


                // Edit Admin pin
                Rectangle {
                    id: editAdminPinID
                    width: button_width
                    height: button_height
                    color: button_color

                    // signal to bring back the button to normal
                    signal deactivate()
                    // signal to take the button to hover/clicked state
                    signal activate()

                    onDeactivate:  {
                        console.log("deactivating edit admin pin")
                        editAdminPinID.color = button_color
                        editAdminPinID.border.color = "gainsboro"
                        editAdminPinButtonID.color = "black"
                        editAdminPinWindow.visible = false
                        mousearea_editAdminPinID.ifClicked = false
                    }

                    onActivate: {
                        console.log("Activating edit admin pin")
                        editAdminPinID.color = clicked_color
                        editAdminPinID.border.color = "black"
                        editAdminPinButtonID.color = "white"
                        editAdminPinWindow.visible = true
                    }

                    Text {
                        id: editAdminPinButtonID
                        text: "Edit Admin Pin"
                        anchors.centerIn:  parent
                    }

                    BMouseArea {
                        id: mousearea_editAdminPinID
                        // Always set the bTextID
                        bTextID: editAdminPinButtonID
                        onClicked: {
                            console.log("Edit Admin pin button clicked")
                            parent.color = clicked_color
                            ifClicked = true
                            bTextID.color = "white"

                            // hide others
                            editNameID.deactivate()
                            editURLID.deactivate()
                            editUserPinID.deactivate()

                            // show us
                            editAdminPinID.activate()


                        }

                    }


                }
            }
        }



        // For the input details
        Rectangle {
            id: cardInputRecID
            Layout.preferredHeight: internal_window_height
            width: 580
            // Here we will have all the rectangles and show only what is required.

            // For Name
            Rectangle {
                id: editNameWindow
                anchors.fill: parent


                Column{
                    id: firstInput
                    anchors.centerIn: parent
                    spacing: 40

                    UInput { id: nameInput }

                    UPassword { id: pinInput }

                    USaveButton {
                        id: saveNameButton
                        onClicked: {
                            console.log("Save button clicked")
                            console.log(nameInput.inputvalue)
                            console.log("Pin: " + pinInput.inputpinvalue)
                        }
                    }

                }


            }

            // For public url
            Rectangle {
                id: editPublicUrlWindow
                anchors.fill: parent
                visible: false

                Column{
                    id: secondInput
                    anchors.centerIn: parent
                    spacing: 40

                    UInput { id: publicURLInput; inputtext: "URL to Public Key" }

                    UPassword { id: pinforURLInput }

                    USaveButton {
                        id: saveURLButton

                        onClicked: {
                            console.log("Save button for URL clicked")
                            console.log(publicURLInput.inputvalue)
                            console.log("Pin: " + pinforURLInput.inputpinvalue)
                        }
                    }

                }
            }

            // For user pin
            Rectangle {
                id: editUserPinWindow
                anchors.fill: parent
                visible: false

                Column{
                    id: thirdInput
                    anchors.centerIn: parent
                    spacing: 40

                    UPassword { id: userPinInput; inputtext: "New User Pin" }

                    UPassword { id: pinforUserPinInput }

                    USaveButton {
                        id: saveUserPinButton

                        onClicked: {
                            console.log("Save button for User pin clicked")
                            console.log(userPinInput.inputpinvalue)
                            console.log("Pin: " + pinforUserPinInput.inputpinvalue)
                        }
                    }

                }
            }

            // For admin pin
            Rectangle {
                id: editAdminPinWindow
                anchors.fill: parent
                visible: false

                Column{
                    id: fourthInput
                    anchors.centerIn: parent
                    spacing: 40

                    UPassword {
                        id: adminPinInput
                        inputtext: "New Admin Pin"
                        KeyNavigation.tab: pinforAdminPinInput.pinbox
                    }

                    UPassword {
                        id: pinforAdminPinInput
                        KeyNavigation.tab: adminPinInput.pinbox
                    }

                    USaveButton {
                        id: saveAdminPinButton


                        onClicked: {
                            console.log("Save button for User pin clicked")
                            console.log(adminPinInput.inputpinvalue)
                            console.log("Pin: " + pinforAdminPinInput.inputpinvalue)
                        }
                    }

                }
            }


        }

    }

    Component.onCompleted:  {
        // Edit Name button should be activate on loading the component
        editNameID.activate()
    }

}
