# Log Extractors

### DNSServer

Use registry:

`HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\DHCPServer\Parameters`
Keys:
* DhcpLogFilePath:
* DhcpV6LogFilePath

### DHCPServer

Use registry:

`HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\DNS\Parameters`
Keys:
* LogFilePath:
* LogFileMaxSize
* LogLevel
RegistrarPaquetes(26), Detalles(25), TCP(16), UDP(15), Entrante(14), Saliente(13), Respuesta(10), Solicitud(9),Actualizaciones(6), Notificaciones(5), Consultas/Transfer(1)
20737   ->00000000000101000100000001-> Saliente, UDP, Consultas/Transfer, Solicitud
24833   ->00000000000110000100000001-> Entrante, UDP, Consultas/Transfer, Solicitud
37121   ->00000000001001000100000001-> Saliente, TCP, Consultas/Transfer, Solicitud
61697   ->00000000001111000100000001-> SalienteyEntrante, TCPyUDP, Consultas/Transfer, Solicitud
61984   ->00000000001111001000100000-> SalienteyEntrante, TCPyUDP, Actualizaciones, Respuesta
61968   ->00000000001111001000010000-> SalienteyEntrante, TCPyUDP, Notificaciones, Respuesta
33616689->10000000001111001100110001-> RegistrarPaquetes, SalienteyEntrante, TCPyUDP, Consultas/TransferyActualizacionesyNotificaciones, SolicitudyRespuesta
50393905->11000000001111001100110001-> RegistrarPaquetes, Detalles, SalienteyEntrante, TCPyUDP, Consultas/TransferyActualizacionesyNotificaciones, SolicitudyRespuesta
* EventLogLevel
0 = Ninguno (000)
1 = Errores (001)
2 = Errores y Advertencias (010)
7 = Todos (111)

