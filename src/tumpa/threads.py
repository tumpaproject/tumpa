import time

import johnnycanencrypt.johnnycanencrypt as rjce
from PySide6.QtCore import QThread, Signal


class HardwareThread(QThread):
    signal = Signal((bool,))
    status_signal = Signal((str,))

    def __init__(self, nextsteps_slot, update_statusbar):
        QThread.__init__(self)
        self.flag = True
        self.signal.connect(nextsteps_slot)
        self.status_signal.connect(update_statusbar)

    def run(self):
        while self.flag:
            time.sleep(1)
            result = rjce.is_smartcard_connected()
            if result:
                card_details = rjce.get_card_details()

                if 'serial_number' in card_details:
                    serial = card_details['serial_number']
                    status_text = f'Yubikey with serial {serial} found'
                else:
                    status_text = 'Yubikey found'
            else:
                status_text = 'No Yubikey found'

            self.status_signal.emit(status_text)
            self.signal.emit(result)
