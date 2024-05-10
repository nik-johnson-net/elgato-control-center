# elgato-control-center

elgato-control-center is a command line tool (CLI) for interacting with the Elgato Control Center application. It provides basic functionality for discovering and controlling lights.


```
elgato-control-center -h
Usage: elgato-control-center <COMMAND>

Commands:
  devices          
  on               
  off              
  set-brightness   
  set-temperature  
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
elgato-control-center devices
id,name
Elgato Key Light Air FFFF (FF:FF:FF:AB:CD:DF),Elgato Key Light Air Right
Elgato Key Light Air FFFE (FF:FF:FF:AB:CD:DE),Elgato Key Light Air Left
```

```bash
# Turn lights on
elgato-control-center on

# Turn lights off
elgato-control-center off

# Turn one light on
elgato-control-center on "Elgato Key Light Air Right"

# Set Temperature to 3000k
elgato-control-center set-temperature 3000

# Set Brightness to 25%
elgato-control-center set-brightness 25
```
