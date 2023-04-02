#!/bin/bash

# Starte pigpiod, falls noch nicht gestartet


# Definiere Pin-Nummer und Service-Name
PIN=3
SERVICE_NAME=start_ledmatrix

# Warte auf Ereignis
while true
do
    if [ $(pigs r $PIN) == "0" ]
    then
        # Überprüfe, ob der Service bereits ausgeführt wird
        if sudo systemctl is-active $SERVICE_NAME > /dev/null
        then
            # Stoppe den Service, wenn er bereits ausgeführt wird
            sudo systemctl stop $SERVICE_NAME
        else
            # Starte den Service, wenn er noch nicht ausgeführt wird
            sudo systemctl start $SERVICE_NAME
        fi

        # Warte, bis Knopf losgelassen wird
        while [ $(pigs r $PIN) == "0" ]
        do
            sleep 0.1
        done
    fi
    sleep 0.1
done