/* hottoh_api web interface: a single page on top of the HTTP API (api/...), without dependencies. */
'use strict';

(() => {
  // ------------------------------------------------------------------ i18n

  const I18N = {
    en: {
      nav_dashboard: 'Stove', nav_schedule: 'Schedule', nav_history: 'History', nav_module: 'Module',
      nav_diagnostics: 'Diagnostics', nav_console: 'Console',
      link_ok: 'Connected', link_down: 'Stove unreachable', link_stale: 'Not updated', link_api: 'hottoh_api not responding', link_searching: 'Searching the stove',
      banner_api: 'hottoh_api does not answer ({error}): check that it is still running.',
      banner_down: 'hottoh_api cannot reach the Wi-Fi module of the stove: {error}',
      banner_stale: 'No news from the stove for {age}: the values shown may be outdated.',
      banner_alarm: 'Stove alarm: {state}',
      theme_auto: 'Theme: automatic', theme_light: 'Theme: light', theme_dark: 'Theme: dark',
      cancel: 'Cancel', confirm: 'Confirm', close: 'Close', refresh: 'Refresh', copy: 'Copy', copied: 'Copied',
      loading: 'Loading…', never: 'never', none: 'none', yes: 'yes', no: 'no', on: 'on', off: 'off',
      enabled: 'enabled', disabled: 'disabled', unknown: 'unknown', ago: '{age} ago',
      feature_off: 'Disabled: set {name} = true in the [features] section of config.ini.',
      write_sent: '{label}…', write_ok: '{label}: done', write_error: '{label}: refused by the stove ({error})',
      write_timeout: '{label}: no answer from the stove', write_failed: '{label}: {error}',
      err_6: 'no record', err_8: 'unknown time zone', err_16: 'data unavailable', err_17: 'value out of range',
      err_18: 'value refused by the module', err_19: 'value refused by the stove board', err_other: 'error {code}',
      // states
      st_off: 'Off', st_starting: 'Starting', st_starting_sub: 'Ignition phase {n} of 7', st_power: 'Heating',
      st_stopping: 'Stopping', st_eco: 'Eco stop', st_low_pellet: 'Pellets low', st_end_pellet: 'Out of pellets',
      st_blackout: 'Power cut', st_antifreeze: 'Anti-freeze', st_ignition_failed: 'Ignition failed',
      st_no_pellet: 'No pellets', st_cover_open: 'Door open', st_alarm: 'Alarm {n}', st_unknown: 'State {n}',
      st_waiting: 'Waiting for the stove',
      // dashboard
      thermostat: 'Thermostat', room: 'Room {n}', setpoint: 'Set point', room_temp: 'Room temperature',
      chrono_follows: 'Chrono mode: the stove follows the weekly schedule{program}.',
      chrono_program: ', {name} now ({temp})',
      turn_on: 'Turn the stove on', turn_off: 'Turn the stove off',
      turn_on_text: 'The stove will start its ignition cycle.', turn_off_text: 'The stove will start its shutdown cycle.',
      power_on_label: 'Stove on', power_off_label: 'Stove off',
      eco_mode: 'Eco mode', eco_desc: 'Stops the stove when the room is warm enough',
      chrono_mode: 'Chrono mode', chrono_desc: 'Follows the weekly schedule',
      power_level: 'Power', power_now: 'now {n}', fan: 'Fan {n}', fan_now: 'actual {n}',
      set_ambiance: 'Room {n} set point {value}', set_power: 'Power {value}', set_fan: 'Fan {n}: {value}',
      set_eco: 'Eco mode {value}', set_chrono: 'Chrono mode {value}', set_program: '{n}: {value}',
      t_room: 'Room', t_room_n: 'Room {n}', t_smoke: 'Smoke', t_water: 'Water', t_puffer: 'Puffer',
      t_boiler: 'Boiler', t_dhw: 'Hot water', t_power: 'Power level', t_updated: 'Updated',
      set_to: 'set point {value}', smoke_fan: 'smoke fan {n}', of_max: 'of {max}',
      // schedule
      schedule_title: 'Weekly schedule', schedule_sub: 'In chrono mode, the stove follows these periods, each with the temperature of its program.',
      programs: 'Chrono programs', program_n: 'Program {n}', no_program: 'No program',
            readonly_hint: 'Read only: chrono_schedule_write is disabled in config.ini.',
      schedule_loading: 'Reading the schedule from the stove (a few seconds)…',
      copy_to: 'Copy the {day} schedule to', weekdays: 'Mon–Fri', weekend: 'Weekend', all_days: 'Every day', copy_apply: 'Copy', copied_days: '{day} copied to {n} day(s): remember to save',
      unsaved: 'Not saved: {days}', save: 'Save to the stove', discard: 'Discard', modified: 'modified',
      discard_title: 'Discard the changes?', discard_text: 'The schedule has unsaved changes.',
      schedule_saved: 'Schedule',       day_0: 'Sunday', day_1: 'Monday', day_2: 'Tuesday', day_3: 'Wednesday', day_4: 'Thursday', day_5: 'Friday', day_6: 'Saturday',
      chrono_off_note: 'Chrono mode is off: the schedule is kept but not followed.',
      // history
      history_title: 'History', history_sub: 'Recorded by the Wi-Fi module every 15 minutes.',
      range_6h: '6 h', range_24h: '24 h', range_3d: '3 days', range_7d: '7 days',
      history_empty: 'No record over this period.',
      records: '{n} records', temperatures: 'Temperatures', power_chart: 'Power level',
      avg: 'average', min: 'min', max: 'max', heating_time: 'Heating time', smoke_max: 'Smoke max',
      alarms: 'Alarms', table_view: 'Table view', time: 'Time', state: 'State',
      // module
      module_title: 'Wi-Fi module', module_sub: 'HottoH Wifier module, clocks, cloud and maintenance.',
      hostname: 'Host name', firmware: 'Firmware', signal: 'Signal', stove_address: 'Address', manufacturer: 'Manufacturer',
      stove_config: 'Stove setup', fans_n: '{n} fan(s)', sensors: 'Sensors',
      fw_uptodate: 'Up to date', fw_update: 'Version {v} available', fw_check: 'Check for updates', fw_checked: 'checked {age}',
      fw_hint: 'The update itself is done with the AppFire application.',
      clocks: 'Clocks', module_clock: 'Module (UTC)', stove_clock: 'Stove', bridge_clock: 'hottoh_api computer',
      offset_module: 'Module offset', offset_hint: 'Module offset from the computer running hottoh_api; stove offset from this device.', offset_stove: 'Stove offset', timezone: 'Time zone',
      tz_unknown: 'not known by the firmware', sync_clock: 'Set to the computer time',
      sync_clock_text: 'The module and stove clocks will be set to the clock of the computer running hottoh_api.',
      apply: 'Apply', tz_apply_text: 'The module will switch to {zone} and set the stove clock.',
      datalog: 'Data logger', first_record: 'Oldest record', last_record: 'Newest record', record_count: 'About {n} records',
      datalog_clear: 'Clear history', datalog_clear_text: 'Every record of the module will be deleted. This cannot be undone.',
      cloud: 'Cloud', relay: 'HottoH relay', cloud_server: '4-noks server', last_upload: 'Last upload',
      pin: 'Relay PIN', show: 'Show', hide: 'Hide', change_pin: 'Change', new_pin: '5 to 10 letters or digits',
      pin_text: 'AppFire in cloud mode keeps the old PIN and will have to be paired again.',
      wifi_networks: 'Wi-Fi networks', wifi_scan: 'Scan', wifi_scan_warn: 'The stove link is suspended for a few seconds during the scan.',
      wifi_scan_text: 'The module suspends its link with the stove for a few seconds. With WPA3 networks nearby, the firmware may restart.',
      ssid: 'SSID', rssi: 'Signal', security: 'Security', bssid: 'BSSID', hidden_ssid: '(hidden)',
      maintenance: 'Maintenance', restart_module: 'Restart the module',
      restart_text: 'The module restarts: stove data is unavailable for about 15 seconds.',
      restart_label: 'Module restart', features: 'Features', features_hint: 'Set in the [features] section of config.ini.',
      // diagnostics
      diag_title: 'Diagnostics', diag_sub: 'Connection to the Wi-Fi module of the stove, hottoh_api counters and raw data.',
      link: 'Connection to the Wi-Fi module', connected: 'Connected', disconnected: 'Disconnected',
      last_response: 'Last answer', connections: 'Connections since start', last_error: 'Last error', connected_since: 'Connected since', error_resolved: 'resolved, connected again since {time}',
      process: 'hottoh_api', version: 'Version', started: 'Started', uptime: 'Uptime', memory: 'Memory',
      threads: 'Threads', fds: 'Open files', pending_writes: 'Commands waiting',
      counters: 'Counters since start', counters_hint: 'A connection lost is a connection that was working; a failed attempt is a connection refused or unanswered (module busy, restarting or off). A few failed attempts right after hottoh_api starts are normal.', latency_avg: 'Average latency', latency_max: 'Max latency',
      c_requests: 'Messages sent', c_answers: 'Answers', c_timeouts: 'Unanswered', c_invalid_frames: 'Invalid messages',
      c_late_answers: 'Late answers', c_decode_errors: 'Decode errors', c_writes_ok: 'Writes ok',
      c_writes_refused: 'Writes refused', c_writes_failed: 'Writes failed', c_reads_ok: 'Reads ok',
      c_reads_refused: 'Reads refused', c_reads_failed: 'Reads failed', c_disconnections: 'Connections lost',
      c_connect_failures: 'Failed connection attempts',
      requests: 'Recent requests', requests_empty: 'No request since start.', id: 'Id', command: 'Command',
      value: 'Value', status: 'Status', attempts: 'Tries', created: 'Created',
      freshness: 'Data pages', raw_data: 'Raw data', snapshot: 'Download a snapshot',
      // console
      console_title: 'Console', console_sub: 'Call any endpoint of the API. Writes act on the stove.',
      endpoint: 'Endpoint…', send: 'Send', body: 'JSON body', response: 'Response', no_response: 'Send a request to see the answer.',
      following: 'Following request {id}…', followed: 'Request {id}: {status}', invalid_json: 'Invalid JSON body: {error}',
      activity: 'Activity', activity_empty: 'Requests sent from this page appear here.', clear: 'Clear',
      console_warn: 'POST requests change the stove settings.',
      prog_name_1: 'Eco', prog_name_2: 'Normal', prog_name_3: 'Comfort', program: 'Program',
      programs_hint: 'The schedule gives each period one of these three programs; change a temperature here and every period using it follows.',
      step_1: 'Set the temperature of the Eco, Normal and Comfort programs.', step_2: 'Pick a day and set its periods, or draw them on the timeline.',
      step_3: 'Copy the day to others if needed, then save to the stove.',
      week_overview: 'Week', overview_hint: 'Click a day to edit it below.', periods_n: '{n} period(s)',
      edit_day: 'Edit {day}', day_detail: '{day}', draw_title: 'Draw on the timeline with', draw_with: 'Program to draw with',
      periods: 'Periods', outside_periods: 'Outside the periods: no program', no_periods: 'No period on this day: the stove follows no program.',
      from: 'from', to: 'to', add_period: 'Add a period', remove: 'Remove',
      alarm_now: 'Alarm in progress', alarm_since: 'since {time}', alarms_title: 'Alarms', alarms_empty: 'No alarm recorded.',
      alarms_hint: 'Alarms seen by hottoh_api, completed by the history of the Wi-Fi module (to the quarter hour) for the times hottoh_api was not running.',
      alarm_live: 'seen by hottoh_api', alarm_logged: 'module history', ongoing: 'in progress',       advice_14: 'The pellet hopper is almost empty: add pellets.',
      advice_15: 'The hopper is empty and the stove stops: fill it, then turn the stove on again.',
      advice_16: 'The stove lost its power supply. Check it restarted, or turn it on again.',
      advice_60: 'The fire did not start. Empty and clean the burn pot, check the pellets, then turn the stove on again.',
      advice_61: 'No pellets reach the burn pot. Fill the hopper, clean the burn pot, then turn the stove on again.',
      advice_69: 'A door of the stove is open: close it.',
      advice_other: 'Alarm reported by the stove (code {n}): check its display and its manual.',
      disc_title: 'Looking for your stove…', disc_text: 'hottoh_api is searching the network {network} for the Wi-Fi module of a HottoH stove.',
      disc_result: 'Last search {time}: {result}. Next search in about a minute.',
      disc_tip_1: 'The stove and this computer must be connected to the same network (same box or router).',
      disc_tip_2: 'The Wi-Fi module accepts a single local connection: close the AppFire application, or switch it to cloud mode.',
      disc_tip_3: 'You can also give the stove address in config.ini: [stove] ip = 192.168.1.50',
      disc_found: 'found on the network', disc_none: 'no HottoH stove found'
    },
    fr: {
      nav_dashboard: 'Poêle', nav_schedule: 'Programmation', nav_history: 'Historique', nav_module: 'Module',
      nav_diagnostics: 'Diagnostic', nav_console: 'Console',
      link_ok: 'Connecté', link_down: 'Poêle injoignable', link_stale: 'Données non actualisées', link_api: 'hottoh_api ne répond pas', link_searching: 'Recherche du poêle',
      banner_api: 'hottoh_api ne répond pas ({error}) : vérifiez qu’il tourne toujours.',
      banner_down: 'hottoh_api n’arrive pas à joindre le module Wi-Fi du poêle : {error}',
      banner_stale: 'Aucune nouvelle du poêle depuis {age} : les valeurs affichées peuvent être anciennes.',
      banner_alarm: 'Alarme du poêle : {state}',
      theme_auto: 'Thème : automatique', theme_light: 'Thème : clair', theme_dark: 'Thème : sombre',
      cancel: 'Annuler', confirm: 'Confirmer', close: 'Fermer', refresh: 'Actualiser', copy: 'Copier', copied: 'Copié',
      loading: 'Chargement…', never: 'jamais', none: 'aucun', yes: 'oui', no: 'non', on: 'activé', off: 'désactivé',
      enabled: 'activée', disabled: 'désactivée', unknown: 'inconnu', ago: 'il y a {age}',
      feature_off: 'Désactivé : mettre {name} = true dans la section [features] de config.ini.',
      write_sent: '{label}…', write_ok: '{label} : fait', write_error: '{label} : refusé par le poêle ({error})',
      write_timeout: '{label} : pas de réponse du poêle', write_failed: '{label} : {error}',
      err_6: 'aucun enregistrement', err_8: 'fuseau inconnu', err_16: 'donnée indisponible', err_17: 'valeur hors limites',
      err_18: 'valeur refusée par le module', err_19: 'valeur refusée par la carte du poêle', err_other: 'erreur {code}',
      st_off: 'Éteint', st_starting: 'Allumage', st_starting_sub: 'Phase d’allumage {n} sur 7', st_power: 'En chauffe',
      st_stopping: 'Extinction', st_eco: 'Arrêt éco', st_low_pellet: 'Granulés bas', st_end_pellet: 'Fin de granulés',
      st_blackout: 'Coupure de courant', st_antifreeze: 'Hors-gel', st_ignition_failed: 'Échec d’allumage',
      st_no_pellet: 'Plus de granulés', st_cover_open: 'Porte ouverte', st_alarm: 'Alarme {n}', st_unknown: 'État {n}',
      st_waiting: 'En attente du poêle',
      thermostat: 'Thermostat', room: 'Pièce {n}', setpoint: 'Consigne', room_temp: 'Température ambiante',
      chrono_follows: 'Mode chrono : le poêle suit la programmation de la semaine{program}.',
      chrono_program: ', {name} en ce moment ({temp})',
      turn_on: 'Allumer le poêle', turn_off: 'Éteindre le poêle',
      turn_on_text: 'Le poêle va lancer son cycle d’allumage.', turn_off_text: 'Le poêle va lancer son cycle d’extinction.',
      power_on_label: 'Allumage du poêle', power_off_label: 'Extinction du poêle',
      eco_mode: 'Mode éco', eco_desc: 'Arrête le poêle quand la pièce est assez chaude',
      chrono_mode: 'Mode chrono', chrono_desc: 'Suit la programmation de la semaine',
      power_level: 'Puissance', power_now: 'actuelle {n}', fan: 'Ventilateur {n}', fan_now: 'réelle {n}',
      set_ambiance: 'Consigne pièce {n} à {value}', set_power: 'Puissance {value}', set_fan: 'Ventilateur {n} : {value}',
      set_eco: 'Mode éco {value}', set_chrono: 'Mode chrono {value}', set_program: '{n} : {value}',
      t_room: 'Pièce', t_room_n: 'Pièce {n}', t_smoke: 'Fumées', t_water: 'Eau', t_puffer: 'Ballon tampon',
      t_boiler: 'Chaudière', t_dhw: 'Eau chaude', t_power: 'Puissance', t_updated: 'Mise à jour',
      set_to: 'consigne {value}', smoke_fan: 'extracteur {n}', of_max: 'sur {max}',
      schedule_title: 'Programmation', schedule_sub: 'En mode chrono, le poêle suit ces périodes, chacune à la température de son programme.',
      programs: 'Programmes chrono', program_n: 'Programme {n}', no_program: 'Aucun programme',
            readonly_hint: 'Lecture seule : chrono_schedule_write est désactivé dans config.ini.',
      schedule_loading: 'Lecture de la programmation dans le poêle (quelques secondes)…',
      copy_to: 'Copier la programmation du {day} sur', weekdays: 'Lun–ven', weekend: 'Week-end', all_days: 'Tous les jours', copy_apply: 'Copier', copied_days: '{day} copié sur {n} jour(s) : pensez à enregistrer',
      unsaved: 'Non enregistré : {days}', save: 'Enregistrer dans le poêle', discard: 'Annuler', modified: 'modifié',
      discard_title: 'Abandonner les modifications ?', discard_text: 'La programmation a des modifications non enregistrées.',
      schedule_saved: 'Programmation',       day_0: 'Dimanche', day_1: 'Lundi', day_2: 'Mardi', day_3: 'Mercredi', day_4: 'Jeudi', day_5: 'Vendredi', day_6: 'Samedi',
      chrono_off_note: 'Le mode chrono est désactivé : la programmation est conservée mais pas suivie.',
      history_title: 'Historique', history_sub: 'Enregistré par le module Wi-Fi toutes les 15 minutes.',
      range_6h: '6 h', range_24h: '24 h', range_3d: '3 jours', range_7d: '7 jours',
      history_empty: 'Aucune mesure sur cette période.',
      records: '{n} mesures', temperatures: 'Températures', power_chart: 'Puissance',
      avg: 'moyenne', min: 'min', max: 'max', heating_time: 'Temps de chauffe', smoke_max: 'Fumées max',
      alarms: 'Alarmes', table_view: 'Vue tableau', time: 'Heure', state: 'État',
      module_title: 'Module Wi-Fi', module_sub: 'Module HottoH Wifier, horloges, cloud et maintenance.',
      hostname: 'Nom d’hôte', firmware: 'Firmware', signal: 'Signal', stove_address: 'Adresse', manufacturer: 'Fabricant',
      stove_config: 'Équipement', fans_n: '{n} ventilateur(s)', sensors: 'Sondes',
      fw_uptodate: 'À jour', fw_update: 'Version {v} disponible', fw_check: 'Rechercher une mise à jour', fw_checked: 'vérifié {age}',
      fw_hint: 'La mise à jour elle-même se fait avec l’application AppFire.',
      clocks: 'Horloges', module_clock: 'Module (UTC)', stove_clock: 'Poêle', bridge_clock: 'Ordinateur hottoh_api',
      offset_module: 'Écart du module', offset_hint: 'Écart du module par rapport à l’ordinateur qui fait tourner hottoh_api ; du poêle par rapport à cet appareil.', offset_stove: 'Écart du poêle', timezone: 'Fuseau horaire',
      tz_unknown: 'inconnu du firmware', sync_clock: 'Régler sur l’heure de l’ordinateur',
      sync_clock_text: 'Les horloges du module et du poêle vont être réglées sur celle de l’ordinateur qui fait tourner hottoh_api.',
      apply: 'Appliquer', tz_apply_text: 'Le module va passer en {zone} et régler l’horloge du poêle.',
      datalog: 'Historique du module', first_record: 'Plus ancienne mesure', last_record: 'Plus récente', record_count: 'Environ {n} mesures',
      datalog_clear: 'Effacer l’historique', datalog_clear_text: 'Toutes les mesures du module seront supprimées, sans retour possible.',
      cloud: 'Cloud', relay: 'Relais HottoH', cloud_server: 'Serveur 4-noks', last_upload: 'Dernier envoi',
      pin: 'PIN du relais', show: 'Afficher', hide: 'Masquer', change_pin: 'Modifier', new_pin: '5 à 10 lettres ou chiffres',
      pin_text: 'AppFire en mode cloud garde l’ancien PIN et devra être réappairée.',
      wifi_networks: 'Réseaux Wi-Fi', wifi_scan: 'Scanner', wifi_scan_warn: 'La liaison avec le poêle est suspendue quelques secondes pendant le scan.',
      wifi_scan_text: 'Le module suspend sa liaison avec le poêle quelques secondes. En présence de réseaux WPA3, le firmware peut redémarrer.',
      ssid: 'SSID', rssi: 'Signal', security: 'Sécurité', bssid: 'BSSID', hidden_ssid: '(masqué)',
      maintenance: 'Maintenance', restart_module: 'Redémarrer le module',
      restart_text: 'Le module redémarre : les données du poêle sont indisponibles une quinzaine de secondes.',
      restart_label: 'Redémarrage du module', features: 'Fonctions', features_hint: 'À régler dans la section [features] de config.ini.',
      diag_title: 'Diagnostic', diag_sub: 'Connexion au module Wi-Fi du poêle, compteurs de hottoh_api et données brutes.',
      link: 'Connexion au module Wi-Fi', connected: 'Connecté', disconnected: 'Déconnecté',
      last_response: 'Dernière réponse', connections: 'Connexions depuis le démarrage', last_error: 'Dernière erreur', connected_since: 'Connecté depuis', error_resolved: 'résolue, reconnecté depuis le {time}',
      process: 'hottoh_api', version: 'Version', started: 'Démarré', uptime: 'Durée de fonctionnement', memory: 'Mémoire',
      threads: 'Threads', fds: 'Fichiers ouverts', pending_writes: 'Commandes en attente',
      counters: 'Compteurs depuis le démarrage', counters_hint: 'Une connexion perdue était établie puis s’est coupée ; une tentative échouée est une connexion refusée ou sans réponse (module occupé, en redémarrage ou éteint). Quelques tentatives échouées juste après le démarrage de hottoh_api sont normales.', latency_avg: 'Latence moyenne', latency_max: 'Latence max',
      c_requests: 'Messages envoyés', c_answers: 'Réponses', c_timeouts: 'Sans réponse', c_invalid_frames: 'Messages invalides',
      c_late_answers: 'Réponses tardives', c_decode_errors: 'Erreurs de décodage', c_writes_ok: 'Écritures réussies',
      c_writes_refused: 'Écritures refusées', c_writes_failed: 'Écritures échouées', c_reads_ok: 'Lectures réussies',
      c_reads_refused: 'Lectures refusées', c_reads_failed: 'Lectures échouées', c_disconnections: 'Connexions perdues',
      c_connect_failures: 'Tentatives de connexion échouées',
      requests: 'Requêtes récentes', requests_empty: 'Aucune requête depuis le démarrage.', id: 'N°', command: 'Commande',
      value: 'Valeur', status: 'Statut', attempts: 'Essais', created: 'Créée',
      freshness: 'Pages de données', raw_data: 'Données brutes', snapshot: 'Télécharger un instantané',
      console_title: 'Console', console_sub: 'Appeler n’importe quel point d’accès de l’API. Les écritures agissent sur le poêle.',
      endpoint: 'Point d’accès…', send: 'Envoyer', body: 'Corps JSON', response: 'Réponse', no_response: 'Envoyez une requête pour voir la réponse.',
      following: 'Suivi de la requête {id}…', followed: 'Requête {id} : {status}', invalid_json: 'Corps JSON invalide : {error}',
      activity: 'Activité', activity_empty: 'Les requêtes envoyées depuis cette page apparaissent ici.', clear: 'Effacer',
      console_warn: 'Les requêtes POST modifient les réglages du poêle.',
      prog_name_1: 'Éco', prog_name_2: 'Normal', prog_name_3: 'Confort', program: 'Programme',
      programs_hint: 'La programmation donne à chaque période l’un de ces trois programmes ; changer une température ici s’applique à toutes les périodes qui l’utilisent.',
      step_1: 'Réglez la température des programmes Éco, Normal et Confort.', step_2: 'Choisissez un jour et définissez ses périodes, ou dessinez-les sur la frise.',
      step_3: 'Copiez le jour sur d’autres si besoin, puis enregistrez dans le poêle.',
      week_overview: 'Semaine', overview_hint: 'Cliquez sur un jour pour le modifier ci-dessous.', periods_n: '{n} période(s)',
      edit_day: 'Modifier le {day}', day_detail: 'Le {day}', draw_title: 'Dessiner sur la frise avec', draw_with: 'Programme pour dessiner',
      periods: 'Périodes', outside_periods: 'En dehors des périodes : aucun programme', no_periods: 'Aucune période ce jour-là : le poêle ne suit aucun programme.',
      from: 'de', to: 'à', add_period: 'Ajouter une période', remove: 'Supprimer',
      alarm_now: 'Alarme en cours', alarm_since: 'depuis {time}', alarms_title: 'Alarmes', alarms_empty: 'Aucune alarme enregistrée.',
      alarms_hint: 'Alarmes relevées par hottoh_api, complétées par l’historique du module Wi-Fi (au quart d’heure près) pour les moments où hottoh_api ne tournait pas.',
      alarm_live: 'relevée par hottoh_api', alarm_logged: 'historique du module', ongoing: 'en cours',       advice_14: 'Le réservoir de granulés est presque vide : rajoutez des granulés.',
      advice_15: 'Le réservoir est vide et le poêle s’arrête : remplissez-le, puis rallumez le poêle.',
      advice_16: 'Le poêle a subi une coupure de courant. Vérifiez qu’il a redémarré, sinon rallumez-le.',
      advice_60: 'Le feu n’a pas pris. Videz et nettoyez le creuset, vérifiez les granulés, puis rallumez le poêle.',
      advice_61: 'Les granulés n’arrivent plus au creuset. Remplissez le réservoir, nettoyez le creuset, puis rallumez le poêle.',
      advice_69: 'Une porte du poêle est ouverte : refermez-la.',
      advice_other: 'Alarme signalée par le poêle (code {n}) : consultez son afficheur et sa notice.',
      disc_title: 'Recherche de votre poêle…', disc_text: 'hottoh_api cherche le module Wi-Fi d’un poêle HottoH sur le réseau {network}.',
      disc_result: 'Dernière recherche {time} : {result}. Nouvelle recherche dans une minute environ.',
      disc_tip_1: 'Le poêle et cet ordinateur doivent être connectés au même réseau (même box).',
      disc_tip_2: 'Le module Wi-Fi n’accepte qu’une seule connexion locale : fermez l’application AppFire, ou passez-la en mode cloud.',
      disc_tip_3: 'Vous pouvez aussi indiquer l’adresse du poêle dans config.ini : [stove] ip = 192.168.1.50',
      disc_found: 'trouvé sur le réseau', disc_none: 'aucun poêle HottoH trouvé'
    }
  };

  const storage = {
    get(key, fallback) {
      try { const v = localStorage.getItem('hottoh.' + key); return v === null ? fallback : JSON.parse(v); } catch { return fallback; }
    },
    set(key, value) {
      try { localStorage.setItem('hottoh.' + key, JSON.stringify(value)); } catch { /* private mode */ }
    }
  };

  let lang = storage.get('lang', (navigator.language || 'en').toLowerCase().startsWith('fr') ? 'fr' : 'en');
  if (!I18N[lang]) lang = 'en';

  function t(key, vars) {
    let text = I18N[lang][key] ?? I18N.en[key] ?? key;
    if (vars) text = text.replace(/\{(\w+)\}/g, (m, name) => (vars[name] ?? m));
    return text;
  }

  // ------------------------------------------------------------------ DOM helpers

  const SVG_NS = 'http://www.w3.org/2000/svg';
  const PROPS = new Set(['value', 'checked', 'disabled', 'selected', 'open', 'tabIndex', 'type']);

  function applyProps(el, props) {
    if (!props) return;
    for (const [key, value] of Object.entries(props)) {
      if (value === undefined || value === null || value === false) continue;
      if (key === 'text') el.textContent = value;
      else if (key === 'class') el.setAttribute('class', value);
      else if (key.startsWith('on') && typeof value === 'function') el.addEventListener(key.slice(2), value);
      else if (key === 'dataset') Object.assign(el.dataset, value);
      else if (key === 'style') Object.assign(el.style, value);
      else if (PROPS.has(key) && !(el instanceof SVGElement)) el[key] = value;
      else el.setAttribute(key, value === true ? '' : value);
    }
  }

  function appendKids(el, kids) {
    for (const kid of kids.flat(Infinity)) {
      if (kid === null || kid === undefined || kid === false) continue;
      el.append(kid instanceof Node ? kid : String(kid));
    }
    return el;
  }

  function h(tag, props, ...kids) {
    const el = document.createElement(tag);
    applyProps(el, props);
    return appendKids(el, kids);
  }

  function s(tag, props, ...kids) {
    const el = document.createElementNS(SVG_NS, tag);
    applyProps(el, props);
    return appendKids(el, kids);
  }

  const ICONS = {
    flame: 'M12 2.5c1 3.2 5.5 5.3 5.5 10a5.5 5.5 0 0 1-11 0c0-2.6 1.4-4.1 2.3-5.2.3 1.7 1.1 2.6 2 2.9-.3-2.6.3-5.3 1.2-7.7z',
    calendar: 'M8 2v4M16 2v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z',
    chart: 'M3 3v18h18M7 15l4-4 3 3 5-6',
    wifi: 'M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M2 9a15 15 0 0 1 20 0M12 20h.01',
    activity: 'M22 12h-4l-3 9L9 3l-3 9H2',
    terminal: 'M4 17l6-6-6-6M12 19h8',
    power: 'M12 3v9M18.4 6.6a9 9 0 1 1-12.8 0',
    refresh: 'M21 12a9 9 0 1 1-2.6-6.4L21 8M21 3v5h-5',
    copy: 'M9 9h11v11H9zM5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1',
    alert: 'M12 9v4M12 17h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z',
    thermo: 'M14 14.8V4a2 2 0 0 0-4 0v10.8a4 4 0 1 0 4 0z',
    wind: 'M17.7 7.7A2.5 2.5 0 1 1 19.5 12H2M9.6 4.6A2 2 0 1 1 11 8H2M12.6 19.4A2 2 0 1 0 14 16H2',
    drop: 'M12 2.7l5.7 5.6a8 8 0 1 1-11.4 0z',
    clock: 'M12 7v5l3 2M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z',
    cloud: 'M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 0 1 0 9z',
    database: 'M3 5c0-1.7 4-3 9-3s9 1.3 9 3-4 3-9 3-9-1.3-9-3zM3 5v14c0 1.7 4 3 9 3s9-1.3 9-3V5M3 12c0 1.7 4 3 9 3s9-1.3 9-3',
    chip: 'M9 3v2M15 3v2M9 19v2M15 19v2M3 9h2M3 15h2M19 9h2M19 15h2M7 5h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z',
    wrench: 'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.8-3.8a6 6 0 0 1-7.9 7.9l-6.9 6.9a2.1 2.1 0 0 1-3-3l6.9-6.9a6 6 0 0 1 7.9-7.9z',
    list: 'M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01',
    send: 'M22 2 11 13M22 2l-7 20-4-9-9-4z',
    download: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3',
    fan: 'M12 12c-3-5 0-9 3-8s2 5-3 8zm0 0c5-3 9 0 8 3s-5 2-8-3zm0 0c3 5 0 9-3 8s-2-5 3-8zm0 0c-5 3-9 0-8-3s5-2 8 3z',
    eye: 'M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z',
    sun: 'M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM12 1v2M12 21v2M4.2 4.2l1.4 1.4M18.4 18.4l1.4 1.4M1 12h2M21 12h2M4.2 19.8l1.4-1.4M18.4 5.6l1.4-1.4',
    moon: 'M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z',
    auto: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 2v20M12 7h5M12 12h7M12 17h5',
    minus: 'M5 12h14', plus: 'M12 5v14M5 12h14',
    trash: 'M3 6h18M8 6V4h8v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6M10 11v6M14 11v6',
    external: 'M15 3h6v6M10 14 21 3M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6'
  };

  function icon(name, props) {
    return s('svg', { viewBox: '0 0 24 24', 'aria-hidden': 'true', fill: 'none', stroke: 'currentColor',
      'stroke-width': 2, 'stroke-linecap': 'round', 'stroke-linejoin': 'round', ...props },
    s('path', { d: ICONS[name] }));
  }

  // ------------------------------------------------------------------ formatting

  const clamp = (v, min, max) => Math.min(max, Math.max(min, v));
  const nf = (digits) => new Intl.NumberFormat(lang, { minimumFractionDigits: digits, maximumFractionDigits: digits });
  const fmtNum = (v, digits = 0) => (v === null || v === undefined || Number.isNaN(v) ? '—' : nf(digits).format(v));
  const fmtTemp = (v) => (v === null || v === undefined ? '—' : fmtNum(v, 1) + ' °C');
  const pad2 = (n) => String(n).padStart(2, '0');

  function fmtDuration(seconds) {
    seconds = Math.max(0, Math.round(seconds));
    const d = Math.floor(seconds / 86400), hh = Math.floor(seconds % 86400 / 3600);
    const mm = Math.floor(seconds % 3600 / 60), ss = seconds % 60;
    if (d) return `${d} j ${hh} h`.replace(' j', lang === 'fr' ? ' j' : ' d');
    if (hh) return `${hh} h ${pad2(mm)}`;
    if (mm) return `${mm} min ${pad2(ss)}`;
    return `${ss} s`;
  }

  function fmtDateTime(date, withSeconds) {
    if (!(date instanceof Date) || Number.isNaN(date.getTime())) return '—';
    return date.toLocaleString(lang, { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit',
      second: withSeconds ? '2-digit' : undefined });
  }

  const parseDate = (text) => (text ? new Date(text) : null);
  /** Current time of the bridge (from its start time and uptime), so that ages do not depend on the clock of this device */
  const bridgeNow = () => (store.status && statusAt
    ? new Date(store.status.started_at).getTime() + store.status.uptime_s * 1000 + (Date.now() - statusAt)
    : Date.now());
  const ageSeconds = (text) => (text ? Math.max(0, (bridgeNow() - new Date(text).getTime()) / 1000) : Infinity);
  const fmtAgo = (text) => (text ? t('ago', { age: fmtDuration(ageSeconds(text)) }) : t('never'));

  function errorText(code) {
    return I18N[lang]['err_' + code] ? t('err_' + code) : t('err_other', { code: code ?? '?' });
  }

  // ------------------------------------------------------------------ API

  async function api(method, path, body) {
    const started = performance.now();
    const init = { method, cache: 'no-store', headers: {} };
    if (body !== undefined) {
      init.headers['Content-Type'] = 'application/json';
      init.body = typeof body === 'string' ? body : JSON.stringify(body);
    }
    try {
      const response = await fetch(path, init);
      const text = await response.text();
      let data = null;
      try { data = text ? JSON.parse(text) : null; } catch { data = text; }
      return { ok: response.ok, status: response.status, data, text, ms: performance.now() - started,
        error: response.ok ? null : (data && data.error) || response.statusText || String(response.status) };
    } catch (error) {
      return { ok: false, status: 0, data: null, text: '', ms: performance.now() - started, error: error.message };
    }
  }

  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  /** Waits for the outcome of a queued request: ok, error, timeout or lost */
  async function followRequest(id, onProgress) {
    const deadline = Date.now() + 75000;
    let last = null;
    while (Date.now() < deadline) {
      const r = await api('GET', `api/request/${id}`);
      if (r.status === 404) return { status: 'lost' };
      if (r.ok) {
        if (r.data.status !== last) { last = r.data.status; onProgress?.(r.data); }
        if (['ok', 'error', 'timeout'].includes(r.data.status)) return r.data;
      }
      await sleep(350);
    }
    return { status: 'timeout' };
  }

  /** Sends a write, follows it and reports the outcome in a toast. Returns the final status. */
  async function command(path, body, label) {
    const toast = showToast(t('write_sent', { label }), 'pending');
    const r = await api('POST', path, body);
    logActivity({ method: 'POST', path, body, status: r.status, ms: r.ms, source: 'ui' });
    if (!r.ok) {
      toast.update(t('write_failed', { label, error: r.error }), 'error');
      return { status: 'failed', error: r.error };
    }
    const outcome = await followRequest(r.data.request_id);
    if (outcome.status === 'ok') toast.update(t('write_ok', { label }), 'ok');
    else if (outcome.status === 'error') toast.update(t('write_error', { label, error: errorText(outcome.error_code) }), 'error');
    else toast.update(t('write_timeout', { label }), 'error');
    refresh();
    return outcome;
  }

  // ------------------------------------------------------------------ activity log

  let activity = storage.get('activity', []);
  const activityListeners = new Set();

  function logActivity(entry) {
    activity.unshift({ at: new Date().toISOString(), ...entry });
    activity = activity.slice(0, 60);
    storage.set('activity', activity);
    activityListeners.forEach((fn) => fn());
  }

  // ------------------------------------------------------------------ toasts & confirm

  function showToast(text, kind = 'info', ms) {
    const el = h('div', { class: `toast ${kind}`, role: 'status' }, h('span', { class: 't-icon' }), h('span', { text }));
    $('#toasts').append(el);
    let timer = null;
    const schedule = (delay) => { clearTimeout(timer); if (delay) timer = setTimeout(() => el.remove(), delay); };
    schedule(kind === 'pending' ? 0 : ms ?? 4000);
    return {
      update(newText, newKind) {
        el.className = `toast ${newKind}`;
        el.lastChild.textContent = newText;
        schedule(newKind === 'error' ? 7000 : 3500);
      }
    };
  }

  function confirmDialog({ title, text, ok, danger }) {
    const dialog = $('#confirm');
    $('#confirm-title').textContent = title;
    $('#confirm-text').textContent = text;
    const okButton = $('#confirm-ok'), cancelButton = $('#confirm-cancel');
    okButton.textContent = ok || t('confirm');
    cancelButton.textContent = t('cancel');
    okButton.className = danger ? 'btn btn-danger btn-solid' : 'btn btn-primary';
    return new Promise((resolve) => {
      const finish = (value) => {
        okButton.onclick = cancelButton.onclick = dialog.oncancel = null;
        if (dialog.open) dialog.close();
        resolve(value);
      };
      okButton.onclick = () => finish(true);
      cancelButton.onclick = () => finish(false);
      dialog.oncancel = () => finish(false);
      dialog.showModal();
      cancelButton.focus();
    });
  }

  function $(selector, root = document) { return root.querySelector(selector); }

  async function copyText(text, button) {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const area = h('textarea', { style: { position: 'fixed', opacity: '0' } });
      area.value = text;
      document.body.append(area);
      area.select();
      document.execCommand('copy');
      area.remove();
    }
    if (button) {
      const previous = button.textContent;
      button.textContent = t('copied');
      setTimeout(() => { button.textContent = previous; }, 1200);
    }
  }

  function downloadJson(name, value) {
    const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' }));
    const link = h('a', { href: url, download: name });
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  // ------------------------------------------------------------------ store & polling

  let statusAt = 0;
  const store = { status: null, inf: null, dat0: null, dat1: null, dat2: null, features: null, alarms: null,
    apiError: null, apiFailures: 0, schedule: null, scheduleAt: 0 };
  const POLL_MS = 2000;
  let pollTimer = null, polling = false, tick = 0;

  const received = (page) => Boolean(page && page.last_updated);

  async function refresh() {
    if (polling) return;
    polling = true;
    try {
      const needInf = !received(store.inf) || tick % 15 === 0;
      const needFeatures = !store.features;
      const previousState = store.dat0?.index_stove_state_raw;
      const [status, dat0, dat1, dat2, inf, features] = await Promise.all([
        api('GET', 'api/status'), api('GET', 'api/dat/0'), api('GET', 'api/dat/1'), api('GET', 'api/dat/2'),
        needInf ? api('GET', 'api/inf') : null, needFeatures ? api('GET', 'api/features') : null
      ]);
      tick++;
      // A single failed request (network hiccup, tab waking up) is not reported
      store.apiFailures = status.ok ? 0 : store.apiFailures + 1;
      store.apiError = store.apiFailures >= 2 ? status.error || 'HTTP ' + status.status : null;
      if (status.ok) { store.status = status.data; statusAt = Date.now(); }
      if (dat0.ok) store.dat0 = dat0.data;
      if (!store.alarms || tick % 5 === 0 || store.dat0?.index_stove_state_raw !== previousState) {
        const alarms = await api('GET', 'api/alarms');
        if (alarms.ok) store.alarms = alarms.data;
      }
      if (dat1.ok) store.dat1 = dat1.data;
      if (dat2.ok) store.dat2 = dat2.data;
      if (inf?.ok) store.inf = inf.data;
      if (features?.ok) store.features = features.data;
    } finally {
      polling = false;
    }
    updateChrome();
    current?.instance.update?.();
  }

  function startPolling() {
    clearTimeout(pollTimer);
    const loop = async () => {
      if (!document.hidden) await refresh();
      pollTimer = setTimeout(loop, POLL_MS);
    };
    loop();
  }

  document.addEventListener('visibilitychange', () => { if (!document.hidden) startPolling(); });

  const feature = (name) => Boolean(store.features && store.features[name]);

  async function loadSchedule(force) {
    if (!force && store.schedule && Date.now() - store.scheduleAt < 10 * 60 * 1000) return store.schedule;
    const r = await api('GET', 'api/chrono/schedule');
    if (!r.ok) throw new Error(r.error);
    store.schedule = r.data.days.map((day) => day.slots.slice());
    store.scheduleAt = Date.now();
    return store.schedule;
  }

  // ------------------------------------------------------------------ stove state

  function stateInfo(dat0) {
    if (!received(dat0)) return { label: t('st_waiting'), tone: 'muted', burning: false };
    const raw = dat0.index_stove_state_raw;
    const info = (key, tone, extra) => ({ label: t(key, { n: raw }), tone, burning: false, raw, ...extra });
    if (raw === 0) return info('st_off', 'muted');
    if (raw >= 1 && raw <= 7) return info('st_starting', 'accent', { burning: true, phase: raw, sub: t('st_starting_sub', { n: raw }) });
    if (raw === 8) return info('st_power', 'accent', { burning: true });
    if (raw === 9 || raw === 10) return info('st_stopping', 'warn', { burning: true });
    if (raw >= 11 && raw <= 13) return info('st_eco', 'info');
    if (raw === 14) return info('st_low_pellet', 'warn', { alarm: true });
    if (raw === 15) return info('st_end_pellet', 'crit', { alarm: true });
    if (raw === 16) return info('st_blackout', 'crit', { alarm: true });
    if (raw === 17) return info('st_antifreeze', 'info');
    if (raw === 60) return info('st_ignition_failed', 'crit', { alarm: true });
    if (raw === 61) return info('st_no_pellet', 'crit', { alarm: true });
    if (raw === 69) return info('st_cover_open', 'crit', { alarm: true });
    if (raw >= 50 && raw <= 99) return info('st_alarm', 'crit', { alarm: true });
    return info('st_unknown', 'muted');
  }

  function stateLabelFromRecord(raw) {
    return stateInfo({ last_updated: 'x', index_stove_state_raw: raw }).label;
  }

  /** Program of the current half hour, from the cached schedule (browser clock) */
  function currentProgram() {
    if (!store.schedule) return null;
    const now = new Date();
    return store.schedule[now.getDay()]?.[now.getHours() * 2 + (now.getMinutes() >= 30 ? 1 : 0)] ?? null;
  }

  const programTemp = (n) => store.dat1?.[`index_program_${n}_temp`];

  // ------------------------------------------------------------------ optimistic controls

  /**
   * Local value shown while a write is on its way: kept until the stove reports it, the write
   * fails, or 15 s have passed.
   */
  function optimistic(read) {
    let desired = null, since = 0, inflight = false;
    return {
      value() {
        if (desired !== null && !inflight) {
          const actual = read();
          if (actual !== undefined && sameValue(actual, desired)) desired = null;
          else if (Date.now() - since > 15000) desired = null;
        }
        return desired !== null ? desired : read();
      },
      pending: () => desired !== null,
      hold(value) { desired = value; since = Date.now(); },
      async run(value, write) {
        desired = value; since = Date.now(); inflight = true;
        current?.instance.update?.();
        const outcome = await write(value);
        inflight = false; since = Date.now();
        if (outcome.status !== 'ok') desired = null;
        current?.instance.update?.();
        return outcome;
      },
      reset() { desired = null; inflight = false; }
    };
  }

  const sameValue = (a, b) => (typeof a === 'number' ? Math.abs(a - b) < 0.05 : a === b);

  function switchControl({ label, desc, onToggle }) {
    const input = h('input', { type: 'checkbox', role: 'switch' });
    const root = h('label', { class: 'switch' },
      h('span', { class: 'switch-label' }, h('span', { text: label }), desc && h('small', { text: desc })),
      input, h('span', { class: 'track', 'aria-hidden': 'true' }));
    input.addEventListener('change', () => {
      const wanted = input.checked;
      input.checked = !wanted; // shown by update() once the write is queued
      onToggle(wanted);
    });
    return {
      el: root,
      set(checked, pending, disabled) {
        input.checked = checked;
        input.disabled = Boolean(disabled);
        root.classList.toggle('pending', Boolean(pending));
      }
    };
  }

  function segmented({ onPick, label }) {
    const root = h('div', { class: 'segmented', role: 'group', 'aria-label': label });
    let key = '';
    return {
      el: root,
      set(values, selected, currentValue, pending, disabled) {
        const newKey = values.join(',');
        if (newKey !== key) {
          key = newKey;
          root.textContent = '';
          for (const v of values) root.append(h('button', { type: 'button', text: v, dataset: { v }, onclick: () => onPick(v) }));
        }
        for (const button of root.children) {
          const v = Number(button.dataset.v);
          button.setAttribute('aria-pressed', String(v === selected));
          button.classList.toggle('current', currentValue !== null && v === currentValue && v !== selected);
          button.classList.toggle('pending', Boolean(pending) && v === selected);
          button.disabled = Boolean(disabled);
        }
      }
    };
  }

  function card(title, iconName, ...kids) {
    const head = h('div', { class: 'card-head' }, h('h2', null, iconName && icon(iconName), title));
    return { el: h('section', { class: 'card' }, head, ...kids), head };
  }

  const featureNote = (name) => h('div', { class: 'note' }, t('feature_off', { name }));

  // ------------------------------------------------------------------ thermostat dial

  function thermostat({ onInput, onCommit }) {
    const size = 300, c = 150, r = 122, sweep = 270, start = 135;
    const arcLength = r * sweep * Math.PI / 180;
    const polar = (deg, radius = r) => [c + radius * Math.cos(deg * Math.PI / 180), c + radius * Math.sin(deg * Math.PI / 180)];
    const [x0, y0] = polar(start), [x1, y1] = polar(start + sweep);
    const arc = `M${x0} ${y0} A${r} ${r} 0 1 1 ${x1} ${y1}`;

    const fill = s('path', { d: arc, class: 'fill', fill: 'none', 'stroke-width': 16, 'stroke-linecap': 'round',
      'stroke-dasharray': `${arcLength} ${arcLength}`, 'stroke-dashoffset': arcLength });
    const knob = s('circle', { r: 13, class: 'knob' });
    const mark = s('circle', { r: 4.5, class: 'current-mark' });
    const ticks = s('g');
    const svgEl = s('svg', { viewBox: `0 0 ${size} ${size}`, 'aria-hidden': 'true' },
      s('defs', null, s('linearGradient', { id: 'thermo-grad', x1: '0', y1: '1', x2: '1', y2: '0' },
        s('stop', { offset: '0', 'stop-color': '#f59f00' }), s('stop', { offset: '1', 'stop-color': '#e8590c' }))),
      s('path', { d: arc, class: 'track', fill: 'none', 'stroke-width': 16, 'stroke-linecap': 'round' }),
      ticks, fill, mark, knob);
    const valueEl = h('div', { class: 'thermo-value num' });
    const setEl = h('div', { class: 'thermo-set' });
    const labelEl = h('div', { class: 'thermo-label', text: t('room_temp') });
    const root = h('div', { class: 'thermo', tabindex: 0, role: 'slider', 'aria-label': t('setpoint') },
      svgEl, h('div', { class: 'thermo-center' }, labelEl, valueEl, setEl));

    let min = 5, max = 35, value = 20, disabled = true, dragging = false, ticksKey = '';
    const fraction = (v) => clamp((v - min) / (max - min || 1), 0, 1);

    function draw() {
      const f = fraction(value);
      fill.setAttribute('stroke-dashoffset', String(arcLength * (1 - f)));
      const [kx, ky] = polar(start + sweep * f);
      knob.setAttribute('cx', kx); knob.setAttribute('cy', ky);
      root.setAttribute('aria-valuemin', min); root.setAttribute('aria-valuemax', max); root.setAttribute('aria-valuenow', value);
    }

    function fromPointer(event) {
      const rect = svgEl.getBoundingClientRect();
      const x = (event.clientX - rect.left) * size / rect.width - c;
      const y = (event.clientY - rect.top) * size / rect.height - c;
      return { distance: Math.hypot(x, y), deg: Math.atan2(y, x) * 180 / Math.PI };
    }

    function valueAt(deg) {
      let rel = ((deg - start) % 360 + 360) % 360;
      if (rel > sweep) rel = rel - sweep < (360 - sweep) / 2 ? sweep : 0;
      return Math.round((min + rel / sweep * (max - min)) * 2) / 2;
    }

    root.addEventListener('pointerdown', (event) => {
      if (disabled) return;
      const p = fromPointer(event);
      if (p.distance < r - 48 || p.distance > r + 34) return;
      dragging = true;
      root.classList.add('dragging');
      try { root.setPointerCapture(event.pointerId); } catch { /* pointer already released */ }
      onInput(valueAt(p.deg));
      event.preventDefault();
    });
    root.addEventListener('pointermove', (event) => { if (dragging) onInput(valueAt(fromPointer(event).deg)); });
    const end = () => { if (dragging) { dragging = false; root.classList.remove('dragging'); onCommit(); } };
    root.addEventListener('pointerup', end);
    root.addEventListener('pointercancel', end);
    root.addEventListener('keydown', (event) => {
      if (disabled) return;
      const step = { ArrowUp: 0.5, ArrowRight: 0.5, ArrowDown: -0.5, ArrowLeft: -0.5 }[event.key];
      if (step) { event.preventDefault(); onInput(value + step); onCommit(); }
    });

    return {
      el: root,
      set({ min: newMin, max: newMax, setpoint, currentTemp, pending, isDisabled }) {
        min = newMin; max = newMax > newMin ? newMax : newMin + 1; disabled = isDisabled;
        if (!dragging || pending) value = setpoint;
        const key = `${min}-${max}`;
        if (key !== ticksKey) {
          ticksKey = key;
          ticks.textContent = '';
          const stepDeg = sweep / 40;
          for (let i = 0; i <= 40; i++) {
            const major = i % 10 === 0;
            const [ax, ay] = polar(start + i * stepDeg, r - 16);
            const [bx, by] = polar(start + i * stepDeg, r - (major ? 24 : 20));
            ticks.append(s('line', { x1: ax, y1: ay, x2: bx, y2: by, class: 'tick', 'stroke-width': major ? 2 : 1 }));
          }
        }
        valueEl.textContent = '';
        valueEl.append(currentTemp === null ? '—' : fmtNum(currentTemp, 1), h('sup', { text: '°' }));
        setEl.textContent = `${t('setpoint')} ${fmtNum(value, 1)} °C`;
        setEl.classList.toggle('pending', Boolean(pending));
        const [mx, my] = polar(start + sweep * fraction(currentTemp ?? min), r - 32);
        mark.setAttribute('cx', mx); mark.setAttribute('cy', my);
        mark.style.display = currentTemp === null ? 'none' : '';
        root.style.opacity = isDisabled ? '0.6' : '';
        draw();
      },
      get value() { return value; },
      preview(v) { value = v; setEl.textContent = `${t('setpoint')} ${fmtNum(v, 1)} °C`; setEl.classList.add('pending'); draw(); }
    };
  }

  // ------------------------------------------------------------------ alarms & discovery panels

  const alarmAdvice = (raw) => (I18N[lang]['advice_' + raw] ? t('advice_' + raw) : t('advice_other', { n: raw }));

  function alarmCard(alarm) {
    return h('section', { class: 'alarm-card', role: 'alert' },
      h('span', { class: 'alarm-icon' }, icon('alert')),
      h('div', null,
        h('div', { class: 'alarm-kicker', text: t('alarm_now') }),
        h('h2', { text: stateLabelFromRecord(alarm.state_raw) }),
        h('p', { text: alarmAdvice(alarm.state_raw) }),
        h('p', { class: 'small', text: t('alarm_since', { time: fmtDateTime(parseDate(alarm.started_at)) }) })));
  }

  function discoveryPanel(discovery) {
    const d = discovery || {};
    return h('section', { class: 'card discovery' },
      h('div', { class: 'discovery-head' }, h('span', { class: 'spinner big' }),
        h('div', null, h('h2', { text: t('disc_title') }), h('p', { class: 'muted', text: t('disc_text', { network: d.network || '…' }) }))),
      d.last_result && !d.searching ? h('p', { class: 'note', text: t('disc_result', { time: fmtAgo(d.last_scan_at),
        result: d.last_result === 'no HottoH module found' ? t('disc_none') : d.last_result }) }) : null,
      h('ul', { class: 'tips' }, ['disc_tip_1', 'disc_tip_2', 'disc_tip_3'].map((key) => h('li', { text: t(key) }))));
  }

  // ------------------------------------------------------------------ view: dashboard

  function mountDashboard(main) {
    let ambiance = 1;
    const setpoint = optimistic(() => store.dat0?.[`index_ambient_t${ambiance}_set`]);
    const onOff = optimistic(() => store.dat0?.index_stove_on);
    const eco = optimistic(() => store.dat0?.index_eco_mode);
    const chrono = optimistic(() => store.dat0?.index_chrono_mode === 2);
    const power = optimistic(() => store.dat0?.index_power_set);
    const fans = [1, 2, 3].map((n) => optimistic(() => store.dat0?.[`index_fan_${n}_set`]));
    let debounce = null, draft = null;

    function limits() {
      const d = store.dat0;
      return d ? { min: d[`index_ambient_t${ambiance}_set_min`], max: d[`index_ambient_t${ambiance}_set_max`] } : { min: 5, max: 35 };
    }

    function commitSetpoint(delay) {
      clearTimeout(debounce);
      if (draft === null) return;
      debounce = setTimeout(() => {
        const value = draft, n = ambiance;
        draft = null;
        setpoint.run(value, (v) => command('api/dat/set_ambiance_temp', { ambiance: n, value: v },
          t('set_ambiance', { n, value: fmtTemp(v) })));
      }, delay);
    }

    function nudge(value) {
      const { min, max } = limits();
      draft = clamp(Math.round(value * 2) / 2, min, max);
      setpoint.hold(draft);
      dial.preview(draft);
    }

    const dial = thermostat({ onInput: nudge, onCommit: () => commitSetpoint(700) });
    const minus = h('button', { class: 'btn btn-round', type: 'button', 'aria-label': '−0.5', onclick: () => { nudge(dial.value - 0.5); commitSetpoint(1000); } }, icon('minus'));
    const plus = h('button', { class: 'btn btn-round', type: 'button', 'aria-label': '+0.5', onclick: () => { nudge(dial.value + 0.5); commitSetpoint(1000); } }, icon('plus'));
    const rangeEl = h('span', { class: 'range' });
    const ambianceSwitch = segmented({ label: t('thermostat'), onPick: (v) => { ambiance = v; setpoint.reset(); draft = null; update(); } });
    const chronoNote = h('div', { class: 'note', hidden: true });
    const thermoCard = card(t('thermostat'), 'thermo', dial.el, h('div', { class: 'thermo-controls' }, minus, rangeEl, plus), chronoNote);
    thermoCard.el.classList.add('thermo-card');
    thermoCard.head.append(ambianceSwitch.el);
    chronoNote.style.marginTop = '14px';

    // Power card
    const powerButton = h('button', { class: 'power-btn', type: 'button', onclick: togglePower }, icon('power'));
    const stateTitle = h('strong'), stateSub = h('span');
    const flame = s('svg', { class: 'flame', viewBox: '0 0 26 32', 'aria-hidden': 'true' },
      s('path', { d: 'M13 1c1.6 5 9 8 9 17a9 9 0 0 1-18 0c0-4.5 2.4-7 3.7-8.8.5 2.8 1.8 4.3 3.3 4.8C10.5 9 11.5 4.6 13 1z' }),
      s('path', { d: 'M13 14c.8 2.5 4.5 4 4.5 8.5a4.5 4.5 0 0 1-9 0c0-2.3 1.2-3.5 1.9-4.4.2 1.4.9 2.2 1.6 2.4-.2-2.3.3-4.4 1-6.5z' }));
    const phaseBar = h('div', { class: 'phase-bar', hidden: true }, [1, 2, 3, 4, 5, 6, 7].map(() => h('span')));
    const ecoSwitch = switchControl({ label: t('eco_mode'), desc: t('eco_desc'),
      onToggle: (v) => eco.run(v, (x) => command('api/dat/set_eco_mode', { value: x }, t('set_eco', { value: t(x ? 'on' : 'off') }))) });
    const chronoSwitch = switchControl({ label: t('chrono_mode'), desc: t('chrono_desc'),
      onToggle: (v) => chrono.run(v, (x) => command('api/dat/set_chrono_mode', { value: x }, t('set_chrono', { value: t(x ? 'on' : 'off') }))) });
    const powerLabel = h('div', { class: 'field-label' }, h('span', { text: t('power_level') }), h('span', { class: 'num' }));
    const powerSeg = segmented({ label: t('power_level'),
      onPick: (v) => power.run(v, (x) => command('api/dat/set_power_level', { value: x }, t('set_power', { value: x }))) });
    const fanBlocks = [1, 2, 3].map((n) => {
      const label = h('div', { class: 'field-label' }, h('span', { text: t('fan', { n }) }), h('span', { class: 'num' }));
      const seg = segmented({ label: t('fan', { n }),
        onPick: (v) => fans[n - 1].run(v, (x) => command('api/dat/set_fan_speed', { fan: n, value: x }, t('set_fan', { n, value: x }))) });
      return { n, label, seg, el: h('div', { hidden: true }, label, seg.el) };
    });
    const powerCard = h('section', { class: 'card power-card' },
      h('div', { class: 'state-row' }, powerButton, h('div', { class: 'state-text' }, stateTitle, stateSub), h('div', { style: { marginLeft: 'auto' } }, flame)),
      phaseBar, h('div', { class: 'divider' }), ecoSwitch.el, chronoSwitch.el, h('div', { class: 'divider' }),
      h('div', null, powerLabel, powerSeg.el), fanBlocks.map((f) => f.el));

    const tiles = h('div', { class: 'tiles' });
    const alarmHolder = h('div');
    const discoveryHolder = h('div');
    const hero = h('div', { class: 'hero' }, thermoCard.el, powerCard);
    main.append(h('div', { class: 'stack' }, discoveryHolder, alarmHolder, hero, tiles));

    async function togglePower() {
      const on = !onOff.value();
      const ok = await confirmDialog({ title: t(on ? 'turn_on' : 'turn_off'), text: t(on ? 'turn_on_text' : 'turn_off_text'),
        ok: t(on ? 'turn_on' : 'turn_off'), danger: !on });
      if (!ok) return;
      onOff.run(on, (v) => command('api/dat/set_on_off', { value: v }, t(v ? 'power_on_label' : 'power_off_label')));
    }

    let scheduleRequested = false;

    function tile(iconName, label, value, sub) {
      return h('div', { class: 'tile' },
        h('div', { class: 'tile-label' }, icon(iconName), label),
        h('div', { class: 'tile-value' }, value),
        sub && h('div', { class: 'tile-sub', text: sub }));
    }

    const tempValue = (v) => [fmtNum(v, 1), h('small', { text: '°C' })];

    function update() {
      const d = store.dat0, d1 = store.dat1, d2 = store.dat2;
      const ready = received(d);
      const searching = store.status && !store.status.stove_address;
      discoveryHolder.textContent = '';
      if (searching) discoveryHolder.append(discoveryPanel(store.status.discovery));
      hero.hidden = tiles.hidden = Boolean(searching);
      alarmHolder.textContent = '';
      if (store.alarms?.current) alarmHolder.append(alarmCard(store.alarms.current));
      const room2 = ready && d.temp_room2_enabled;
      if (!room2) ambiance = 1;
      ambianceSwitch.el.hidden = !room2;
      ambianceSwitch.set([1, 2], ambiance, null, false, false);
      if (room2) [...ambianceSwitch.el.children].forEach((b) => { b.textContent = t('room', { n: b.dataset.v }); });

      const { min, max } = limits();
      const sp = setpoint.value();
      dial.set({ min, max, setpoint: sp ?? min, currentTemp: ready ? d[`index_ambient_t${ambiance}`] : null,
        pending: setpoint.pending(), isDisabled: !ready });
      minus.disabled = plus.disabled = !ready;
      rangeEl.textContent = ready ? `${fmtNum(min, 0)} – ${fmtNum(max, 0)} °C` : '';

      const chronoOn = chrono.value();
      if (ready && chronoOn && !scheduleRequested && feature('chrono_schedule_read')) {
        scheduleRequested = true;
        loadSchedule().then(update, () => {});
      }
      chronoNote.hidden = !(ready && chronoOn);
      if (ready && chronoOn) {
        const program = currentProgram();
        chronoNote.textContent = t('chrono_follows', { program: program ? t('chrono_program', { name: t('prog_name_' + program), temp: fmtTemp(programTemp(program)) }) : '' });
      }

      const state = stateInfo(d);
      const on = onOff.value();
      powerButton.classList.toggle('on', Boolean(on));
      powerButton.classList.toggle('pending', onOff.pending());
      powerButton.disabled = !ready;
      powerButton.setAttribute('aria-label', t(on ? 'turn_off' : 'turn_on'));
      powerButton.title = t(on ? 'turn_off' : 'turn_on');
      stateTitle.textContent = state.label;
      stateSub.textContent = state.sub || (ready ? `${t('t_power')} ${d.index_power_level} · ${fmtAgo(d.last_updated)}` : '');
      flame.classList.toggle('burning', state.burning);
      phaseBar.hidden = !state.phase;
      [...phaseBar.children].forEach((el, i) => el.classList.toggle('done', i < (state.phase || 0)));

      ecoSwitch.set(Boolean(eco.value()), eco.pending(), !ready);
      chronoSwitch.set(Boolean(chronoOn), chrono.pending(), !ready);

      if (ready) {
        const values = [];
        for (let v = d.index_power_min; v <= d.index_power_max && values.length < 20; v++) values.push(v);
        powerSeg.set(values, power.value(), state.burning ? d.index_power_level : null, power.pending(), false);
        powerLabel.lastChild.textContent = state.burning ? t('power_now', { n: d.index_power_level }) : '';
        for (const f of fanBlocks) {
          const maxSpeed = d[`index_fan_${f.n}_set_max`];
          f.el.hidden = !(maxSpeed > 0);
          if (maxSpeed > 0) {
            const values = Array.from({ length: Math.min(maxSpeed, 20) + 1 }, (_, i) => i);
            const actual = d2?.[`index_fan_${f.n}_speed`];
            f.seg.set(values, fans[f.n - 1].value(), null, fans[f.n - 1].pending(), false);
            f.label.lastChild.textContent = actual !== undefined ? t('fan_now', { n: actual }) : '';
          }
        }
      }

      tiles.textContent = '';
      if (!ready) return;
      tiles.append(tile('thermo', d.temp_room2_enabled ? t('t_room_n', { n: 1 }) : t('t_room'), tempValue(d.index_ambient_t1),
        t('set_to', { value: fmtTemp(d.index_ambient_t1_set) })));
      if (d.temp_room2_enabled) tiles.append(tile('thermo', t('t_room_n', { n: 2 }), tempValue(d.index_ambient_t2), t('set_to', { value: fmtTemp(d.index_ambient_t2_set) })));
      if (d.temp_room3_enabled && received(d2)) tiles.append(tile('thermo', t('t_room_n', { n: 3 }), tempValue(d2.index_room_temp_3), t('set_to', { value: fmtTemp(d2.index_room_temp_3_set) })));
      tiles.append(tile('wind', t('t_smoke'), tempValue(d.index_smoke_t), d.index_fan_smoke ? t('smoke_fan', { n: d.index_fan_smoke }) : null));
      if (d.temp_water_enabled) tiles.append(tile('drop', t('t_water'), tempValue(d.index_water), t('set_to', { value: fmtTemp(d.index_water_set) })));
      if (d.boiler_enabled && received(d2)) {
        tiles.append(tile('drop', t('t_puffer'), tempValue(d2.index_puffer), t('set_to', { value: fmtTemp(d2.index_puffer_set) })));
        tiles.append(tile('drop', t('t_boiler'), tempValue(d2.index_boiler), t('set_to', { value: fmtTemp(d2.index_boiler_set) })));
      }
      if (d.domestic_hot_water_enabled && received(d2)) tiles.append(tile('drop', t('t_dhw'), tempValue(d2.index_dhw), t('set_to', { value: fmtTemp(d2.index_dhw_set) })));
      tiles.append(tile('flame', t('t_power'), [String(d.index_power_level), h('small', { text: ' / ' + d.index_power_max })], t('set_to', { value: d.index_power_set })));
      if (received(d1) && chronoOn) {
        const program = currentProgram();
        if (program) tiles.append(tile('calendar', t('prog_name_' + program), tempValue(programTemp(program)), `${t('program_n', { n: program })} · ${t('chrono_mode')}`));
      }
    }

    return { update, destroy: () => clearTimeout(debounce) };
  }

  // ------------------------------------------------------------------ view: schedule

  const DISPLAY_DAYS = [1, 2, 3, 4, 5, 6, 0];
  const DAY_NAMES = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];
  const SLOTS = 48;
  const slotTime = (i) => (i >= SLOTS ? '24:00' : `${pad2(Math.floor(i / 2))}:${i % 2 ? '30' : '00'}`);
  const dayName = (d) => t('day_' + d);

  /** Periods of consecutive half hours running the same program (slots without program left out) */
  function slotsToRanges(slots) {
    const ranges = [];
    for (let i = 0; i < SLOTS;) {
      let j = i;
      while (j < SLOTS && slots[j] === slots[i]) j++;
      if (slots[i]) ranges.push({ start: i, end: j, program: slots[i] });
      i = j;
    }
    return ranges;
  }

  function rangesToSlots(ranges) {
    const slots = new Array(SLOTS).fill(0);
    for (const range of ranges) for (let k = range.start; k < range.end; k++) slots[k] = range.program;
    return slots;
  }

  function mountSchedule(main) {
    const programs = [1, 2, 3].map((n) => optimistic(() => store.dat1?.[`index_program_${n}_temp`]));
    const chrono = optimistic(() => store.dat0?.index_chrono_mode === 2);
    const debounces = [null, null, null], drafts = [null, null, null];
    let base = null, work = null, loadError = null, reloadStarted = false;
    let day = new Date().getDay(), brush = 3, painting = false, lastPainted = null;
    const copyTargets = new Set();

    main.append(h('h1', { class: 'section-title', text: t('schedule_title') }), h('p', { class: 'section-sub', text: t('schedule_sub') }));

    // Programs and chrono mode
    const chronoSwitch = switchControl({ label: t('chrono_mode'), desc: t('chrono_desc'),
      onToggle: (v) => chrono.run(v, (x) => command('api/dat/set_chrono_mode', { value: x }, t('set_chrono', { value: t(x ? 'on' : 'off') }))) });
    const programEls = [1, 2, 3].map((n) => {
      const temp = h('div', { class: 'p-temp' });
      const step = (delta) => {
        const d1 = store.dat1;
        if (!received(d1)) return;
        const minT = d1[`index_program_${n}_temp_min`], maxT = d1[`index_program_${n}_temp_max`];
        drafts[n - 1] = clamp(Math.round(((drafts[n - 1] ?? programs[n - 1].value()) + delta) * 2) / 2, minT, maxT);
        programs[n - 1].hold(drafts[n - 1]);
        update();
        clearTimeout(debounces[n - 1]);
        debounces[n - 1] = setTimeout(() => {
          const value = drafts[n - 1];
          drafts[n - 1] = null;
          programs[n - 1].run(value, (v) => command('api/dat/set_chrono_temp', { chrono: n, value: v }, t('set_program', { n: programName(n), value: fmtTemp(v) })));
        }, 1000);
      };
      const el = h('div', { class: 'program' },
        h('span', { class: 'swatch', style: { background: `var(--p${n})` } }),
        h('div', null, h('div', { class: 'p-name' }, h('b', { text: t('prog_name_' + n) }), ` · ${t('program_n', { n })}`), temp),
        h('div', { class: 'p-ctl' },
          h('button', { class: 'btn', type: 'button', 'aria-label': '−0.5', onclick: () => step(-0.5) }, icon('minus')),
          h('button', { class: 'btn', type: 'button', 'aria-label': '+0.5', onclick: () => step(0.5) }, icon('plus'))));
      return { el, temp };
    });
    const programsCard = card(t('programs'), 'thermo', chronoSwitch.el,
      h('p', { class: 'muted small', style: { margin: '4px 0 12px' }, text: t('programs_hint') }),
      h('div', { class: 'programs' }, programEls.map((p) => p.el)));

    const steps = h('ol', { class: 'steps' }, ['step_1', 'step_2', 'step_3'].map((key, i) =>
      h('li', null, h('span', { class: 'step-num', text: i + 1 }), h('span', { text: t(key) }))));

    const overviewCard = card(t('week_overview'), 'calendar');
    const reloadButton = h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: () => reload(true) }, icon('refresh'), t('refresh'));
    overviewCard.head.append(reloadButton);
    const overviewBody = h('div', { class: 'stack' });
    overviewCard.el.append(overviewBody);

    const editorCard = card('', 'clock');
    const editorBody = h('div', { class: 'stack' });
    editorCard.el.append(editorBody);

    const saveText = h('span');
    const saveBar = h('div', { class: 'savebar', hidden: true }, saveText,
      h('div', { class: 'row' },
        h('button', { class: 'btn btn-ghost btn-sm', type: 'button', text: t('discard'), onclick: () => { work = base.map((d) => d.slice()); renderAll(); } }),
        h('button', { class: 'btn btn-primary btn-sm', type: 'button', text: t('save'), onclick: save })));

    main.append(h('div', { class: 'stack' }, programsCard.el, steps, overviewCard.el, editorCard.el, saveBar));

    const writable = () => feature('chrono_schedule_write');
    const programName = (p) => (p ? t('prog_name_' + p) : t('no_program'));
    const changedDays = () => (base && work ? DISPLAY_DAYS.filter((d) => work[d].some((p, i) => p !== base[d][i])) : []);

    async function reload(force) {
      if (!feature('chrono_schedule_read')) { renderAll(); return; }
      if (force && changedDays().length && !(await confirmDialog({ title: t('discard_title'), text: t('discard_text') }))) return;
      base = work = null; loadError = null;
      renderAll();
      try {
        base = (await loadSchedule(force)).map((d) => d.slice());
        work = base.map((d) => d.slice());
      } catch (error) {
        loadError = error.message;
      }
      renderAll();
    }

    function renderAll() {
      renderOverview();
      renderEditor();
      updateSaveBar();
    }

    /** One bar of 48 half hours */
    function bar(slots, className, dayIndex) {
      return h('div', { class: className, dataset: { day: dayIndex } }, slots.map((p, i) =>
        h('span', { class: 'slot' + (base && p !== base[dayIndex][i] ? ' changed' : ''), dataset: { i, p },
          title: `${slotTime(i)}–${slotTime(i + 1)} · ${programName(p)}` })));
    }

    function renderOverview() {
      overviewBody.textContent = '';
      if (!feature('chrono_schedule_read')) { overviewBody.append(featureNote('chrono_schedule_read')); return; }
      if (loadError) { overviewBody.append(h('div', { class: 'note warn' }, icon('alert', { width: 18, height: 18 }), loadError)); return; }
      if (!work) { overviewBody.append(h('div', { class: 'loading' }, h('span', { class: 'spinner' }), t('schedule_loading'))); return; }
      if (store.dat0 && store.dat0.index_chrono_mode !== 2) overviewBody.append(h('div', { class: 'note', text: t('chrono_off_note') }));
      overviewBody.append(h('div', { class: 'legend' }, [1, 2, 3, 0].map((p) =>
        h('span', null, h('i', { class: 'box', style: { background: `var(--p${p})` } }), p ? `${programName(p)} · ${fmtTemp(programTemp(p))}` : programName(p)))));
      const now = new Date();
      const rows = h('div', { class: 'overview' },
        h('div', { class: 'overview-row overview-hours' }, h('span'),
          h('div', { class: 'hours' }, [0, 6, 12, 18].map((hour) => h('span', { text: `${pad2(hour)}:00` })), h('span', { text: '24:00' }))));
      for (const d of DISPLAY_DAYS) {
        const summary = slotsToRanges(work[d]).length;
        const row = h('button', { class: 'overview-row' + (d === day ? ' selected' : ''), type: 'button',
          'aria-pressed': String(d === day), onclick: () => selectDay(d, true) },
        h('span', { class: 'overview-day' + (d === now.getDay() ? ' today' : '') }, dayName(d),
          changedDays().includes(d) ? h('i', { class: 'dirty-dot', title: t('modified') }) : null),
        h('div', { class: 'overview-bar-wrap' }, bar(work[d], 'overview-bar', d),
          d === now.getDay() ? h('span', { class: 'now-line', style: { left: `${(now.getHours() * 60 + now.getMinutes()) / 1440 * 100}%` } }) : null),
        h('span', { class: 'overview-count muted small', text: t('periods_n', { n: summary }) }));
        rows.append(row);
      }
      overviewBody.append(rows, h('p', { class: 'muted small', text: t('overview_hint') }));
    }

    function selectDay(d, scroll) {
      day = d;
      copyTargets.clear();
      renderAll();
      if (scroll && matchMedia('(max-width: 820px)').matches) editorCard.el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }

    function setDaySlots(slots) {
      work[day] = slots;
      renderAll();
    }

    function timeSelect(value, from, to, onChange) {
      const select = h('select', { class: 'time-select', onchange: () => onChange(Number(select.value)) });
      for (let i = from; i <= to; i++) select.append(h('option', { value: i, text: slotTime(i), selected: i === value }));
      select.disabled = !writable();
      return select;
    }

    function renderEditor() {
      const title = editorCard.el.querySelector('h2');
      title.lastChild.textContent = t(writable() ? 'edit_day' : 'day_detail', { day: dayName(day).toLowerCase() });
      editorBody.textContent = '';
      if (!work) { editorCard.el.hidden = true; return; }
      editorCard.el.hidden = false;

      editorBody.append(h('div', { class: 'segmented day-tabs', role: 'group' }, DISPLAY_DAYS.map((d) =>
        h('button', { type: 'button', text: dayName(d).slice(0, 3), 'aria-pressed': String(d === day), title: dayName(d), onclick: () => selectDay(d, false) }))));

      // Timeline to draw on
      const timeline = bar(work[day], 'timeline' + (writable() ? '' : ' readonly'), day);
      const timelineWrap = h('div', { class: 'timeline-wrap' },
        h('div', { class: 'timeline-hours' }, Array.from({ length: 13 }, (_, k) => h('span', { text: pad2(k * 2) }))), timeline);
      if (writable()) {
        const brushes = h('div', { class: 'brushes', role: 'group', 'aria-label': t('draw_with') }, [3, 2, 1, 0].map((p) => h('button', {
          class: 'brush', type: 'button', 'aria-pressed': String(brush === p), dataset: { p },
          onclick: () => { brush = p; [...brushes.children].forEach((b) => b.setAttribute('aria-pressed', String(Number(b.dataset.p) === brush))); }
        }, h('i', { style: { background: `var(--p${p})` } }), programName(p))));
        editorBody.append(h('div', null, h('div', { class: 'field-label' }, h('span', { text: t('draw_title') })), brushes));
        const paint = (el) => {
          const i = Number(el.dataset.i);
          const from = lastPainted ?? i;
          lastPainted = i;
          const cells = timeline.children;
          for (let k = Math.min(from, i); k <= Math.max(from, i); k++) {
            work[day][k] = brush;
            cells[k].dataset.p = brush;
            cells[k].classList.toggle('changed', base[day][k] !== brush);
          }
        };
        timeline.addEventListener('pointerdown', (event) => {
          const slot = event.target.closest('.slot');
          if (!slot) return;
          painting = true;
          lastPainted = null;
          try { timeline.setPointerCapture(event.pointerId); } catch { /* pointer already released */ }
          paint(slot);
          event.preventDefault();
        });
        timeline.addEventListener('pointermove', (event) => {
          if (!painting) return;
          const slot = document.elementFromPoint(event.clientX, event.clientY)?.closest('.slot');
          if (slot && timeline.contains(slot)) paint(slot);
        });
        const stop = () => { if (painting) { painting = false; renderAll(); } };
        timeline.addEventListener('pointerup', stop);
        timeline.addEventListener('pointercancel', stop);
      }
      editorBody.append(timelineWrap);

      // Periods as a list
      const ranges = slotsToRanges(work[day]);
      const list = h('div', { class: 'periods' });
      const apply = (index, change) => {
        const edited = { ...ranges[index], ...change };
        if (edited.end <= edited.start) edited.end = Math.min(SLOTS, edited.start + 1);
        const others = ranges.filter((_, k) => k !== index);
        // The edited period is applied last: it takes over the half hours it now covers
        setDaySlots(rangesToSlots([...others, edited]));
      };
      if (!ranges.length) list.append(h('div', { class: 'note', text: t('no_periods') }));
      ranges.forEach((range, index) => {
        const programSelect = h('select', { class: 'program-select', 'aria-label': t('program'), onchange: () => apply(index, { program: Number(programSelect.value) }) },
          [3, 2, 1].map((p) => h('option', { value: p, text: `${programName(p)} · ${fmtTemp(programTemp(p))}`, selected: p === range.program })));
        programSelect.disabled = !writable();
        list.append(h('div', { class: 'period' },
          h('span', { class: 'swatch', style: { background: `var(--p${range.program})` } }),
          h('span', { class: 'period-times' },
            h('span', { class: 'muted small', text: t('from') }), timeSelect(range.start, 0, SLOTS - 1, (v) => apply(index, { start: v })),
            h('span', { class: 'muted small', text: t('to') }), timeSelect(range.end, 1, SLOTS, (v) => apply(index, { end: v }))),
          programSelect,
          writable() && h('button', { class: 'btn btn-sm btn-ghost', type: 'button', 'aria-label': t('remove'), title: t('remove'),
            onclick: () => setDaySlots(rangesToSlots(ranges.filter((_, k) => k !== index))) }, icon('trash'))));
      });
      editorBody.append(h('div', null, h('div', { class: 'field-label' }, h('span', { text: t('periods') }), h('span', { text: t('outside_periods') })), list));
      if (writable()) {
        editorBody.append(h('div', { class: 'row' }, h('button', { class: 'btn btn-sm', type: 'button', onclick: () => {
          const last = ranges[ranges.length - 1];
          const start = last ? Math.min(last.end, SLOTS - 1) : 14;
          setDaySlots(rangesToSlots([...ranges, { start, end: Math.min(SLOTS, start + 4), program: 3 }]));
        } }, icon('plus'), t('add_period'))));

        // Copy to other days
        const chips = h('div', { class: 'day-chips' });
        const renderChips = () => {
          chips.textContent = '';
          for (const d of DISPLAY_DAYS.filter((x) => x !== day)) {
            chips.append(h('button', { type: 'button', class: 'chip', 'aria-pressed': String(copyTargets.has(d)), text: dayName(d).slice(0, 3), title: dayName(d),
              onclick: () => { copyTargets.has(d) ? copyTargets.delete(d) : copyTargets.add(d); renderChips(); } }));
          }
          copyButton.disabled = copyTargets.size === 0;
        };
        const pick = (days) => { copyTargets.clear(); days.filter((d) => d !== day).forEach((d) => copyTargets.add(d)); renderChips(); };
        const copyButton = h('button', { class: 'btn btn-sm btn-primary', type: 'button', onclick: () => {
          for (const d of copyTargets) work[d] = work[day].slice();
          const count = copyTargets.size;
          copyTargets.clear();
          renderAll();
          showToast(t('copied_days', { day: dayName(day), n: count }), 'ok');
        } }, icon('copy'), t('copy_apply'));
        renderChips();
        editorBody.append(h('div', { class: 'copy-box' },
          h('div', { class: 'field-label' }, h('span', { text: t('copy_to', { day: dayName(day).toLowerCase() }) })),
          chips,
          h('div', { class: 'row' },
            h('button', { class: 'btn btn-sm btn-ghost', type: 'button', text: t('weekdays'), onclick: () => pick([1, 2, 3, 4, 5]) }),
            h('button', { class: 'btn btn-sm btn-ghost', type: 'button', text: t('weekend'), onclick: () => pick([6, 0]) }),
            h('button', { class: 'btn btn-sm btn-ghost', type: 'button', text: t('all_days'), onclick: () => pick(DISPLAY_DAYS) }),
            h('span', { style: { flex: 1 } }), copyButton)));
      } else {
        editorBody.append(h('div', { class: 'note', text: t('readonly_hint') }));
      }
    }

    function updateSaveBar() {
      const days = changedDays();
      saveBar.hidden = days.length === 0;
      saveText.textContent = t('unsaved', { n: days.length, days: days.map((d) => dayName(d).slice(0, 3)).join(', ') });
    }

    async function save() {
      const days = changedDays();
      if (!days.length) return;
      saveBar.classList.add('busy');
      const snapshot = work.map((d) => d.slice());
      const outcome = await command('api/chrono/schedule', { days: days.map((d) => ({ day: DAY_NAMES[d], slots: snapshot[d] })) },
        `${t('schedule_saved')} (${days.map(dayName).join(', ')})`);
      saveBar.classList.remove('busy');
      if (outcome.status === 'ok') {
        base = snapshot.map((d) => d.slice());
        store.schedule = snapshot.map((d) => d.slice());
        store.scheduleAt = Date.now();
        renderAll();
      }
    }

    function update() {
      const ready = received(store.dat1);
      chronoSwitch.set(Boolean(chrono.value()), chrono.pending(), !received(store.dat0));
      programEls.forEach((p, i) => {
        p.temp.textContent = ready ? fmtTemp(programs[i].value()) : '—';
        p.temp.classList.toggle('pending', programs[i].pending());
      });
      if (!base && !loadError && store.features && !reloadStarted) { reloadStarted = true; reload(false); }
    }

    const beforeUnload = (event) => { if (changedDays().length) { event.preventDefault(); event.returnValue = ''; } };
    window.addEventListener('beforeunload', beforeUnload);
    renderAll();
    return {
      update,
      destroy() { window.removeEventListener('beforeunload', beforeUnload); debounces.forEach(clearTimeout); },
      dirty: () => changedDays().length > 0
    };
  }

  // ------------------------------------------------------------------ charts

  function niceTicks(min, max, count = 5, integer = false) {
    if (min === max) { if (min === 0) max = 1; else { min -= 1; max += 1; } }
    const raw = (max - min) / count;
    const magnitude = 10 ** Math.floor(Math.log10(raw));
    let step = [1, 2, 2.5, 5, 10].map((m) => m * magnitude).find((st) => st >= raw) || raw;
    if (integer) step = Math.max(1, Math.ceil(step));
    const lo = Math.floor(min / step) * step, hi = Math.ceil(max / step) * step;
    const ticks = [];
    for (let v = lo; v <= hi + step / 2; v += step) ticks.push(Math.round(v * 1000) / 1000);
    return ticks;
  }

  /**
   * Time series chart with one y axis. `series`: [{ label, color, values }]; `times`: UTC seconds;
   * `step` draws steps (levels) instead of lines; `tooltipExtra(i)` adds rows to the tooltip.
   */
  function timeChart(container, { times, series, unit, height = 240, step = false, area = false, yMin, tooltipExtra, digits = 1 }) {
    const draw = () => {
      container.textContent = '';
      const width = Math.max(320, container.clientWidth);
      const m = { l: 44, r: series.length > 1 ? 70 : 14, t: 12, b: 26 };
      const all = series.flatMap((se) => se.values).filter((v) => v !== null && v !== undefined);
      if (!all.length || times.length < 1) return;
      let lo = Math.min(...all), hi = Math.max(...all);
      if (yMin !== undefined) lo = Math.min(lo, yMin);
      const yt = niceTicks(lo, hi, 4, digits === 0);
      const y0 = yt[0], y1 = yt[yt.length - 1];
      const t0 = times[0], t1 = times[times.length - 1] === t0 ? t0 + 900 : times[times.length - 1];
      const x = (tm) => m.l + (tm - t0) / (t1 - t0) * (width - m.l - m.r);
      const y = (v) => m.t + (1 - (v - y0) / (y1 - y0 || 1)) * (height - m.t - m.b);
      const root = s('svg', { viewBox: `0 0 ${width} ${height}`, height, role: 'img' });

      for (const v of yt) {
        root.append(s('line', { x1: m.l, x2: width - m.r, y1: y(v), y2: y(v), class: 'grid-line' }));
        root.append(s('text', { x: m.l - 8, y: y(v) + 4, 'text-anchor': 'end', class: 'axis-text', text: fmtNum(v, Number.isInteger(v) ? 0 : 1) }));
      }
      const span = t1 - t0;
      // Enough room for each label (about 56 px)
      const maxLabels = Math.max(2, Math.floor((width - m.l - m.r) / 56));
      const hourStep = [1, 2, 3, 6, 12, 24, 48].find((st) => span / 3600 / st <= maxLabels) || 48;
      const first = new Date(t0 * 1000);
      first.setMinutes(0, 0, 0);
      while (first.getHours() % hourStep) first.setHours(first.getHours() + 1);
      for (let d = new Date(first); d.getTime() / 1000 <= t1; d.setHours(d.getHours() + hourStep)) {
        const tm = d.getTime() / 1000;
        if (tm < t0) continue;
        const label = hourStep >= 24 || (d.getHours() === 0 && hourStep >= 6)
          ? d.toLocaleDateString(lang, { weekday: 'short', day: 'numeric' })
          : `${pad2(d.getHours())}:00`;
        root.append(s('line', { x1: x(tm), x2: x(tm), y1: height - m.b, y2: height - m.b + 4, class: 'grid-line' }));
        root.append(s('text', { x: x(tm), y: height - 6, 'text-anchor': 'middle', class: 'axis-text', text: label }));
      }

      const gap = 1900; // more than two missed records: break the line
      const endLabels = [];
      for (const se of series) {
        // Runs of consecutive points, split on missing values and gaps in the records
        const runs = [];
        let lastIndex = -1;
        se.values.forEach((v, i) => {
          if (v === null || v === undefined) { lastIndex = -1; return; }
          if (lastIndex < 0 || times[i] - times[lastIndex] > gap) runs.push([]);
          runs[runs.length - 1].push([x(times[i]), y(v)]);
          lastIndex = i;
        });
        const line = (run) => run.map(([px, py], k) => (k === 0 ? `M${px} ${py}` : step ? `H${px}V${py}` : `L${px} ${py}`)).join('');
        if (area) {
          const base = y(clamp(0, y0, y1));
          const d = runs.map((run) => `M${run[0][0]} ${base}V${run[0][1]}${line(run).replace(/^M[^HVL]*/, '')}V${base}Z`).join('');
          root.append(s('path', { d, class: 'area', fill: se.color, stroke: 'none' }));
        }
        root.append(s('path', { d: runs.map(line).join(''), class: 'series', stroke: se.color }));
        lastIndex = se.values.findLastIndex((v) => v !== null && v !== undefined);
        if (lastIndex >= 0 && series.length > 1) endLabels.push({ y: y(se.values[lastIndex]), text: se.label });
      }
      endLabels.sort((a, b) => a.y - b.y);
      for (let i = 1; i < endLabels.length; i++) endLabels[i].y = Math.max(endLabels[i].y, endLabels[i - 1].y + 14);
      for (const label of endLabels) root.append(s('text', { x: width - m.r + 8, y: label.y + 4, class: 'end-label', text: label.text }));

      const cross = s('line', { y1: m.t, y2: height - m.b, class: 'crosshair', visibility: 'hidden' });
      const dots = series.map((se) => s('circle', { r: 4.5, fill: se.color, class: 'hover-dot', visibility: 'hidden' }));
      root.append(cross, ...dots);
      const overlay = s('rect', { x: m.l, y: m.t, width: Math.max(0, width - m.l - m.r), height: height - m.t - m.b, fill: 'transparent', tabindex: 0 });
      root.append(overlay);
      const tip = h('div', { class: 'tooltip', hidden: true });
      container.append(root, tip);

      let index = -1;
      const show = (i) => {
        index = clamp(i, 0, times.length - 1);
        const px = x(times[index]);
        cross.setAttribute('x1', px); cross.setAttribute('x2', px); cross.setAttribute('visibility', 'visible');
        dots.forEach((dot, k) => {
          const v = series[k].values[index];
          dot.setAttribute('visibility', v === null || v === undefined ? 'hidden' : 'visible');
          if (v !== null && v !== undefined) { dot.setAttribute('cx', px); dot.setAttribute('cy', y(v)); }
        });
        tip.textContent = '';
        tip.append(h('div', { class: 'tt-title', text: fmtDateTime(new Date(times[index] * 1000)) }));
        for (const se of series) {
          const v = se.values[index];
          tip.append(h('div', { class: 'tt-row' }, h('i', { style: { background: se.color } }),
            h('b', { text: v === null || v === undefined ? '—' : fmtNum(v, digits) + (unit ? ' ' + unit : '') }), h('span', { text: se.label })));
        }
        for (const [label, value] of tooltipExtra?.(index) || []) {
          tip.append(h('div', { class: 'tt-row' }, h('i'), h('b', { text: value }), h('span', { text: label })));
        }
        tip.hidden = false;
        const scale = container.clientWidth / width;
        const left = px * scale;
        const tipWidth = tip.offsetWidth;
        tip.style.left = `${left + 14 + tipWidth > container.clientWidth ? left - tipWidth - 14 : left + 14}px`;
        tip.style.top = `${m.t}px`;
      };
      const hide = () => { cross.setAttribute('visibility', 'hidden'); dots.forEach((d) => d.setAttribute('visibility', 'hidden')); tip.hidden = true; };
      overlay.addEventListener('pointermove', (event) => {
        const rect = root.getBoundingClientRect();
        const tm = t0 + ((event.clientX - rect.left) * width / rect.width - m.l) / (width - m.l - m.r) * (t1 - t0);
        let best = 0;
        for (let i = 1; i < times.length; i++) if (Math.abs(times[i] - tm) < Math.abs(times[best] - tm)) best = i;
        show(best);
      });
      overlay.addEventListener('pointerleave', hide);
      overlay.addEventListener('blur', hide);
      overlay.addEventListener('focus', () => show(times.length - 1));
      overlay.addEventListener('keydown', (event) => {
        const delta = { ArrowLeft: -1, ArrowRight: 1 }[event.key];
        if (delta) { event.preventDefault(); show((index < 0 ? times.length - 1 : index) + delta); }
      });
    };
    draw();
    const observer = new ResizeObserver(() => { if (Math.abs(container.clientWidth - lastWidth) > 4) { lastWidth = container.clientWidth; draw(); } });
    let lastWidth = container.clientWidth;
    observer.observe(container);
    return () => observer.disconnect();
  }

  // ------------------------------------------------------------------ view: history

  const RANGES = [['range_6h', 6], ['range_24h', 24], ['range_3d', 72], ['range_7d', 168]];
  const RECORD_STEP = 900;
  const DATALOG_PAGE = 100;

  /**
   * Records of the data logger already read, kept in the browser: they never change, so a visit
   * only asks the module for the missing ones. `empty` lists the periods known to have no record.
   */
  function datalogCache() {
    const key = 'datalog:' + (store.status?.stove_address || '');
    if (datalogCache.key !== key) {
      const saved = storage.get(key, null);
      datalogCache.key = key;
      datalogCache.records = new Map((saved?.records || []).map((r) => [r.utc, r]));
      datalogCache.empty = saved?.empty || [];
    }
    return datalogCache;
  }

  function saveDatalogCache() {
    const cache = datalogCache();
    const oldest = Date.now() / 1000 - 8 * 86400;
    const records = [...cache.records.values()].filter((r) => r.utc >= oldest).sort((a, b) => a.utc - b.utc);
    cache.empty = cache.empty.filter(([, end]) => end >= oldest).slice(-50);
    storage.set(cache.key, { records, empty: cache.empty });
  }

  function clearDatalogCache() {
    const cache = datalogCache();
    cache.records.clear();
    cache.empty = [];
    saveDatalogCache();
  }

  const datalogRange = (from, to) => [...datalogCache().records.values()].filter((r) => r.utc >= from && r.utc <= to).sort((a, b) => a.utc - b.utc);

  /** Periods of [from, to] not covered by the cache */
  function missingPeriods(from, to) {
    const cache = datalogCache();
    const known = datalogRange(from, to).map((r) => r.utc);
    const gaps = [];
    let covered = from - 1;
    for (const utc of known) {
      if (utc - covered > 2 * RECORD_STEP) gaps.push([covered + 1, utc - 1]);
      covered = utc;
    }
    // The newest record may not be written yet
    if (to - covered > RECORD_STEP + 120) gaps.push([covered + 1, to]);
    return gaps.filter(([start, end]) => !cache.empty.some(([a, b]) => a <= start && b >= end));
  }

  async function fetchDatalog(from, to, onPage) {
    const cache = datalogCache();
    for (const [start, end] of missingPeriods(from, to)) {
      let next = start, got = 0;
      for (let page = 0; page < 20 && next <= end; page++) {
        const count = clamp(Math.ceil((end - next) / RECORD_STEP) + 1, 1, DATALOG_PAGE);
        const r = await api('GET', `api/datalog?from=${next}&count=${count}`);
        if (!r.ok) throw new Error(r.error);
        const list = r.data.records;
        for (const record of list) cache.records.set(record.utc, record);
        got += list.filter((record) => record.utc <= end).length;
        onPage?.();
        if (list.length < count || !list.length || list[list.length - 1].utc >= end) break;
        next = list[list.length - 1].utc + 1;
      }
      if (!got && end < Date.now() / 1000 - 2 * RECORD_STEP) cache.empty.push([start, end]);
    }
    saveDatalogCache();
  }

  /** Alarms of the module history: consecutive records in the same alarm state */
  function alarmsFromRecords(records) {
    const events = [];
    for (const record of records) {
      const last = events[events.length - 1];
      if (StoveAlarm(record.state_raw)) {
        if (last && last.state_raw === record.state_raw && record.utc - last.lastUtc <= 2 * RECORD_STEP) last.lastUtc = record.utc;
        else events.push({ state_raw: record.state_raw, startUtc: record.utc, lastUtc: record.utc });
      }
    }
    return events;
  }

  const StoveAlarm = (raw) => (raw >= 14 && raw <= 16) || (raw >= 50 && raw <= 99);

  function mountHistory(main) {
    let hours = storage.get('historyHours', 24), records = null, loading = false, error = null, progress = 0, cleanups = [];
    const presets = h('div', { class: 'segmented', role: 'group' });
    const refreshButton = h('button', { class: 'btn btn-sm', type: 'button', onclick: () => load() }, icon('refresh'), t('refresh'));
    const info = h('span', { class: 'muted small' });
    const body = h('div', { class: 'stack' });
    main.append(h('h1', { class: 'section-title', text: t('history_title') }), h('p', { class: 'section-sub', text: t('history_sub') }),
      h('div', { class: 'filters' }, presets, refreshButton, info), body);

    function renderPresets() {
      presets.textContent = '';
      for (const [key, value] of RANGES) {
        presets.append(h('button', { type: 'button', text: t(key), 'aria-pressed': String(value === hours),
          onclick: () => { hours = value; storage.set('historyHours', hours); renderPresets(); load(); } }));
      }
    }

    async function load() {
      if (loading || !feature('datalog_read')) return;
      loading = true; error = null;
      const to = Math.floor(Date.now() / 1000), from = to - hours * 3600;
      const show = () => {
        records = datalogRange(from, to);
        render();
      };
      if (datalogCache().records.size) show();
      else render();
      body.querySelectorAll('.chart').forEach((c) => c.classList.add('refreshing'));
      info.textContent = t('loading');
      try {
        await fetchDatalog(from, to, show);
      } catch (e) {
        error = e.message;
      }
      loading = false;
      show();
    }

    function stat(label, value, sub) {
      return h('div', { class: 'tile' }, h('div', { class: 'tile-label', text: label }), h('div', { class: 'tile-value', text: value }), sub && h('div', { class: 'tile-sub', text: sub }));
    }

    function render() {
      cleanups.forEach((fn) => fn());
      cleanups = [];
      body.textContent = '';
      if (!feature('datalog_read')) { body.append(featureNote('datalog_read')); info.textContent = ''; return; }
      if (error) body.append(h('div', { class: 'note warn' }, icon('alert', { width: 18, height: 18 }), error));
      if (!records) { if (loading) body.append(h('div', { class: 'loading' }, h('span', { class: 'spinner' }), t('loading'))); return; }
      if (!loading) info.textContent = records.length ? `${t('records', { n: records.length })} · ${fmtDateTime(new Date(records[0].utc * 1000))} → ${fmtDateTime(new Date(records[records.length - 1].utc * 1000))}` : '';
      if (!records.length) { body.append(h('div', { class: 'card empty', text: t('history_empty') })); return; }

      const d0 = store.dat0 || {};
      const times = records.map((r) => r.utc);
      const rooms = records.map((r) => r.room_temp);
      const burning = records.filter((r) => r.state_raw >= 1 && r.state_raw <= 10).length;
      const alarms = new Set(records.filter((r) => stateInfo({ last_updated: 'x', index_stove_state_raw: r.state_raw }).alarm).map((r) => r.state_raw));
      const avg = rooms.reduce((a, b) => a + b, 0) / rooms.length;
      body.append(h('div', { class: 'tiles' },
        stat(t('t_room'), fmtTemp(avg), `${t('min')} ${fmtTemp(Math.min(...rooms))} · ${t('max')} ${fmtTemp(Math.max(...rooms))}`),
        stat(t('heating_time'), fmtDuration(burning * 900)),
        stat(t('smoke_max'), fmtTemp(Math.max(...records.map((r) => r.smoke_temp)))),
        stat(t('alarms'), String(alarms.size), [...alarms].map(stateLabelFromRecord).join(', ') || null)));
      body.append(alarmHistoryCard(records));

      const css = getComputedStyle(document.documentElement);
      const color = (name) => css.getPropertyValue(name).trim();
      const stateRow = (i) => [[t('state'), stateLabelFromRecord(records[i].state_raw)], [t('t_power'), String(records[i].power_level)]];
      // One chart per measure: room and smoke temperatures do not share a useful scale
      const charts = [
        { title: t('t_room'), icon: 'thermo', height: 220, series: [{ label: t('t_room'), color: color('--series-1'), values: rooms }], unit: '°C' },
        d0.temp_water_enabled && { title: t('t_water'), icon: 'drop', height: 180, series: [{ label: t('t_water'), color: color('--series-3'), values: records.map((r) => r.water_temp) }], unit: '°C' },
        { title: t('t_smoke'), icon: 'wind', height: 180, series: [{ label: t('t_smoke'), color: color('--series-2'), values: records.map((r) => r.smoke_temp) }], unit: '°C' },
        { title: t('power_chart'), icon: 'flame', height: 150, step: true, area: true, yMin: 0, digits: 0, series: [{ label: t('t_power'), color: color('--accent'), values: records.map((r) => r.power_level) }] }
      ].filter(Boolean);
      for (const chart of charts) {
        const holder = h('div', { class: 'chart' });
        body.append(card(chart.title, chart.icon, holder).el);
        cleanups.push(timeChart(holder, { times, ...chart, tooltipExtra: (i) => stateRow(i).slice(0, chart.step ? 1 : 2) }));
      }
      const table = h('table', null,
        h('thead', null, h('tr', null, [t('time'), t('state'), t('t_power'), t('t_room'), t('t_smoke'), d0.temp_water_enabled && t('t_water')].filter(Boolean).map((x) => h('th', { text: x })))),
        h('tbody', null, records.slice().reverse().map((r) => h('tr', null,
          h('td', { text: fmtDateTime(new Date(r.utc * 1000)) }), h('td', { text: stateLabelFromRecord(r.state_raw) }),
          h('td', { text: r.power_level }), h('td', { text: fmtTemp(r.room_temp) }), h('td', { text: fmtTemp(r.smoke_temp) }),
          d0.temp_water_enabled && h('td', { text: fmtTemp(r.water_temp) })))));
      body.append(h('details', { class: 'raw' }, h('summary', null, h('span', { text: t('table_view') })),
        h('div', { class: 'table-wrap', style: { maxHeight: '420px', padding: '0 8px 8px' } }, table)));
    }

    function alarmHistoryCard(list) {
      const live = (store.alarms?.events || []).map((e) => ({ state_raw: e.state_raw, start: parseDate(e.started_at), end: parseDate(e.ended_at), source: 'live' }));
      // Module history adds the alarms hottoh_api did not see
      const logged = alarmsFromRecords(list)
        .map((e) => ({ state_raw: e.state_raw, start: new Date(e.startUtc * 1000), end: new Date((e.lastUtc + RECORD_STEP) * 1000), source: 'logged' }))
        .filter((e) => !live.some((l) => l.state_raw === e.state_raw && l.start - 30 * 60000 <= e.end && (l.end || new Date()) >= e.start - 30 * 60000));
      const events = [...live, ...logged].sort((a, b) => b.start - a.start);
      const c = card(t('alarms_title'), 'alert');
      c.el.append(h('p', { class: 'muted small', style: { marginBottom: '10px' }, text: t('alarms_hint') }));
      if (!events.length) { c.el.append(h('div', { class: 'empty', text: t('alarms_empty') })); return c.el; }
      c.el.append(h('div', { class: 'alarm-list' }, events.slice(0, 30).map((e) => h('div', { class: 'alarm-row' },
        h('span', { class: `pill ${e.end ? 'pill-muted' : 'pill-crit'}`, text: e.end ? fmtDuration((e.end - e.start) / 1000) : t('ongoing') }),
        h('div', null,
          h('b', { text: stateLabelFromRecord(e.state_raw) }),
          h('div', { class: 'muted small', text: `${fmtDateTime(e.start)} · ${t(e.source === 'live' ? 'alarm_live' : 'alarm_logged')}` }),
          h('div', { class: 'small', text: alarmAdvice(e.state_raw) }))))));
      return c.el;
    }

    renderPresets();
    let started = false;
    return {
      update() { if (!started && store.features) { started = true; render(); load(); } },
      destroy() { cleanups.forEach((fn) => fn()); }
    };
  }

  // ------------------------------------------------------------------ view: module

  function mountModule(main) {
    main.append(h('h1', { class: 'section-title', text: t('module_title') }), h('p', { class: 'section-sub', text: t('module_sub') }));
    const grid = h('div', { class: 'grid grid-2' });
    main.append(grid);
    let built = false;
    const kv = (rows) => h('dl', { class: 'kv' }, rows.filter(Boolean).map(([k, v]) => [h('dt', { text: k }), h('dd', null, v ?? '—')]));
    const loadingEl = () => h('div', { class: 'loading' }, h('span', { class: 'spinner' }), t('loading'));
    const errorEl = (message) => h('div', { class: 'note warn' }, icon('alert', { width: 18, height: 18 }), message);

    /** Card whose content comes from an async loader, with a refresh button */
    function asyncCard(title, iconName, featureName, loader) {
      const c = card(title, iconName);
      const content = h('div', { class: 'stack' });
      c.el.append(content);
      const reloadCard = async (...args) => {
        content.textContent = '';
        content.append(loadingEl());
        try {
          const nodes = await loader(reloadCard, ...args);
          content.textContent = '';
          appendKids(content, [nodes]);
        } catch (error) {
          content.textContent = '';
          content.append(errorEl(error.message));
        }
      };
      if (featureName && !feature(featureName)) content.append(featureNote(featureName));
      else {
        c.head.append(h('button', { class: 'btn btn-sm btn-ghost', type: 'button', title: t('refresh'), 'aria-label': t('refresh'), onclick: () => reloadCard() }, icon('refresh')));
        reloadCard();
      }
      return c.el;
    }

    const get = async (path) => {
      const r = await api('GET', path);
      if (!r.ok) throw new Error(r.error);
      return r.data;
    };

    function signalBars(level) {
      return h('span', { class: 'signal', 'aria-hidden': 'true' }, [1, 2, 3, 4].map((i) => h('i', { class: i <= level ? 'on' : '', style: { height: `${i * 3 + 2}px` } })));
    }

    function build() {
      built = true;
      const inf = store.inf || {}, d0 = store.dat0 || {};

      // Module & firmware
      const moduleCard = card(t('module_title'), 'chip');
      const fwLine = h('span', { class: 'row' });
      const fwCheck = async (refreshNow) => {
        if (!feature('firmware_update_check')) return;
        fwLine.textContent = '';
        fwLine.append(h('span', { class: 'spinner' }));
        try {
          const fw = await get('api/firmware' + (refreshNow ? '?refresh=true' : ''));
          fwLine.textContent = '';
          fwLine.append(fw.update_available
            ? h('span', { class: 'pill pill-warn', text: t('fw_update', { v: fw.latest_available }) })
            : h('span', { class: 'pill pill-good', text: t('fw_uptodate') }),
          h('span', { class: 'muted small', text: t('fw_checked', { age: fmtAgo(fw.checked_at) }) }));
        } catch (error) {
          fwLine.textContent = '';
          fwLine.append(h('span', { class: 'muted small', text: error.message }));
        }
      };
      const sensors = [d0.temp_room1_enabled && t('t_room_n', { n: 1 }), d0.temp_room2_enabled && t('t_room_n', { n: 2 }),
        d0.temp_room3_enabled && t('t_room_n', { n: 3 }), d0.temp_water_enabled && t('t_water'),
        d0.boiler_enabled && t('t_boiler'), d0.domestic_hot_water_enabled && t('t_dhw')].filter(Boolean);
      moduleCard.el.append(kv([
        [t('hostname'), inf.hostname], [t('firmware'), h('span', { class: 'row' }, h('b', { text: inf.version || '—' }), fwLine)],
        [t('signal'), inf.signal], [t('stove_address'), h('span', null, h('span', { class: 'mono', text: store.status?.stove_address || '—' }),
          store.status?.discovery ? h('span', { class: 'muted small', text: ` · ${t('disc_found')}` }) : null)],
        [t('manufacturer'), d0.index_manufacturer], [t('stove_config'), received(d0) ? `${t('fans_n', { n: d0.fan_number })} · ${sensors.join(', ') || t('none')}` : null]
      ]));
      if (feature('firmware_update_check')) {
        moduleCard.el.append(h('div', { class: 'spread', style: { marginTop: '14px' } }, h('span', { class: 'muted small', text: t('fw_hint') }),
          h('button', { class: 'btn btn-sm', type: 'button', onclick: () => fwCheck(true) }, icon('refresh'), t('fw_check'))));
        fwCheck(false);
      }
      grid.append(moduleCard.el);

      // Clocks & time zone
      grid.append(asyncCard(t('clocks'), 'clock', 'clock_read', async (reloadCard) => {
        const [clock, tz] = await Promise.all([get('api/clock'), feature('timezone_read') ? get('api/timezone') : null]);
        const stoveDate = parseDate(clock.stove_time);
        const stoveOffset = stoveDate ? Math.round((stoveDate.getTime() - Date.now()) / 1000) : null;
        const offsetText = (sec) => (sec === null || sec === undefined ? '—' : `${sec > 0 ? '+' : ''}${fmtNum(sec)} s`);
        const offsetPill = (sec) => h('span', { class: `pill ${sec !== null && Math.abs(sec) > 120 ? 'pill-warn' : 'pill-muted'}`, text: offsetText(sec) });
        const nodes = [kv([
          [t('module_clock'), fmtDateTime(parseDate(clock.module_time), true)],
          [t('stove_clock'), fmtDateTime(stoveDate, true)],
          [t('bridge_clock'), fmtDateTime(new Date(clock.bridge_utc * 1000), true)],
          [t('offset_module'), offsetPill(clock.module_offset_s)],
          [t('offset_stove'), offsetPill(stoveOffset)],
          tz && [t('timezone'), h('span', { class: 'row' }, h('b', { text: tz.zone }), !tz.known && h('span', { class: 'pill pill-warn', text: t('tz_unknown') }))]
        ])];
        nodes.push(h('p', { class: 'muted small', text: t('offset_hint') }));
        const actions = h('div', { class: 'row' });
        if (tz && feature('timezone_write') && Array.isArray(tz.available)) {
          const select = h('select', { 'aria-label': t('timezone') }, tz.available.map((z) => h('option', { value: z, text: z, selected: z === tz.zone })));
          actions.append(select, h('button', { class: 'btn btn-sm', type: 'button', text: t('apply'), onclick: async () => {
            const zone = select.value;
            if (!(await confirmDialog({ title: t('timezone'), text: t('tz_apply_text', { zone }) }))) return;
            if ((await command('api/timezone', { zone }, `${t('timezone')} ${zone}`)).status === 'ok') setTimeout(reloadCard, 1500);
          } }));
        }
        if (feature('clock_write')) {
          actions.append(h('button', { class: 'btn btn-sm', type: 'button', onclick: async () => {
            if (!(await confirmDialog({ title: t('sync_clock'), text: t('sync_clock_text') }))) return;
            if ((await command('api/clock', {}, t('sync_clock'))).status === 'ok') setTimeout(reloadCard, 1500);
          } }, icon('clock'), t('sync_clock')));
        }
        if (actions.childElementCount) nodes.push(actions);
        return nodes;
      }));

      // Data logger
      grid.append(asyncCard(t('datalog'), 'database', 'datalog_read', async (reloadCard) => {
        const bounds = await get('api/datalog/info');
        const count = bounds.last_utc && bounds.first_utc ? Math.round((bounds.last_utc - bounds.first_utc) / 900) + 1 : 0;
        const nodes = [kv([
          [t('first_record'), fmtDateTime(parseDate(bounds.first_time))],
          [t('last_record'), fmtDateTime(parseDate(bounds.last_time))],
          ['', h('span', { class: 'muted', text: t('record_count', { n: fmtNum(count) }) })]
        ]), h('a', { href: '#/history', class: 'small', text: t('history_title') + ' →' })];
        if (feature('datalog_clear')) {
          nodes.push(h('div', null, h('button', { class: 'btn btn-sm btn-danger', type: 'button', onclick: async () => {
            if (!(await confirmDialog({ title: t('datalog_clear'), text: t('datalog_clear_text'), ok: t('datalog_clear'), danger: true }))) return;
            if ((await command('api/datalog/clear', {}, t('datalog_clear'))).status === 'ok') { clearDatalogCache(); setTimeout(reloadCard, 1500); }
          } }, t('datalog_clear'))));
        }
        return nodes;
      }));

      // Cloud & PIN
      grid.append(asyncCard(t('cloud'), 'cloud', 'cloud_read', async () => {
        const cloud = await get('api/cloud');
        const rows = [
          [t('relay'), h('span', { class: 'mono', text: `${cloud.relay_balancer.host}:${cloud.relay_balancer.port}` })],
          [t('cloud_server'), h('span', { class: 'mono', text: `${cloud.cloud_server.url}:${cloud.cloud_server.port}${cloud.cloud_server.path}` })],
          [t('last_upload'), cloud.cloud_last_upload_time ? fmtDateTime(parseDate(cloud.cloud_last_upload_time)) : t('never')]
        ];
        const nodes = [kv(rows)];
        if (feature('pin_read') || feature('pin_write')) {
          const pinValue = h('span', { class: 'mono', text: '••••••' });
          const pinRow = h('div', { class: 'row' }, h('span', { class: 'muted', text: t('pin') }), pinValue);
          if (feature('pin_read')) {
            const showButton = h('button', { class: 'btn btn-sm btn-ghost', type: 'button' }, icon('eye'), t('show'));
            let shown = false;
            showButton.onclick = async () => {
              if (shown) { pinValue.textContent = '••••••'; shown = false; showButton.lastChild.textContent = t('show'); return; }
              showButton.disabled = true;
              try {
                pinValue.textContent = (await get('api/pin')).pin;
                shown = true;
                showButton.lastChild.textContent = t('hide');
              } catch (error) { showToast(error.message, 'error'); }
              showButton.disabled = false;
            };
            pinRow.append(showButton);
          }
          nodes.push(h('div', { class: 'divider' }), pinRow);
          if (feature('pin_write')) {
            const input = h('input', { type: 'text', placeholder: t('new_pin'), maxlength: 10, autocomplete: 'off', pattern: '[A-Za-z0-9]{5,10}' });
            nodes.push(h('div', { class: 'row' }, input, h('button', { class: 'btn btn-sm', type: 'button', text: t('change_pin'), onclick: async () => {
              const pin = input.value.trim();
              if (!/^[A-Za-z0-9]{5,10}$/.test(pin)) { showToast(t('new_pin'), 'error'); return; }
              if (!(await confirmDialog({ title: t('pin'), text: t('pin_text') }))) return;
              if ((await command('api/pin', { pin }, t('pin'))).status === 'ok') input.value = '';
            } })));
          }
        }
        return nodes;
      }));

      // Wi-Fi scan
      const wifiCard = card(t('wifi_networks'), 'wifi');
      const wifiBody = h('div', { class: 'stack' });
      wifiCard.el.append(wifiBody);
      if (!feature('wifi_scan')) wifiBody.append(featureNote('wifi_scan'));
      else {
        const scanButton = h('button', { class: 'btn btn-sm', type: 'button', onclick: scan }, icon('wifi'), t('wifi_scan'));
        wifiCard.head.append(scanButton);
        wifiBody.append(h('div', { class: 'note', text: t('wifi_scan_warn') }));
        async function scan() {
          if (!(await confirmDialog({ title: t('wifi_scan'), text: t('wifi_scan_text') }))) return;
          scanButton.disabled = true;
          wifiBody.textContent = '';
          wifiBody.append(loadingEl());
          try {
            const networks = await get('api/wifi/scan');
            wifiBody.textContent = '';
            wifiBody.append(h('div', { class: 'table-wrap' }, h('table', null,
              h('thead', null, h('tr', null, [t('ssid'), t('rssi'), t('security'), t('bssid')].map((x) => h('th', { text: x })))),
              h('tbody', null, networks.map((n) => h('tr', null,
                h('td', { class: 'wrap', text: n.ssid || t('hidden_ssid') }),
                h('td', null, signalBars(n.rssi >= -55 ? 4 : n.rssi >= -67 ? 3 : n.rssi >= -78 ? 2 : 1), `${n.rssi} dBm`),
                h('td', { text: n.security }), h('td', { class: 'mono', text: n.bssid })))))));
          } catch (error) {
            wifiBody.textContent = '';
            wifiBody.append(errorEl(error.message));
          }
          scanButton.disabled = false;
        }
      }
      grid.append(wifiCard.el);

      // Maintenance & features
      const maintenance = card(t('maintenance'), 'wrench');
      if (feature('module_restart')) {
        maintenance.el.append(h('div', { class: 'spread' }, h('span', { class: 'muted small', text: t('restart_text') }),
          h('button', { class: 'btn btn-danger', type: 'button', onclick: async () => {
            if (!(await confirmDialog({ title: t('restart_module'), text: t('restart_text'), ok: t('restart_module'), danger: true }))) return;
            command('api/module/restart', {}, t('restart_label'));
          } }, icon('power'), t('restart_module'))));
      } else {
        maintenance.el.append(featureNote('module_restart'));
      }
      grid.append(maintenance.el);
      const featuresCard = card(t('features'), 'list',
        h('p', { class: 'muted small', style: { marginBottom: '10px' }, text: t('features_hint') }),
        h('div', { class: 'feature-list' }, Object.entries(store.features || {}).map(([name, on]) =>
          h('div', { class: 'feature' }, h('code', { text: name }), h('span', { class: `pill ${on ? 'pill-good' : 'pill-muted'}`, text: t(on ? 'enabled' : 'disabled') })))));
      featuresCard.el.classList.add('span-all');
      grid.append(featuresCard.el);
    }

    return { update() { if (!built && store.features && received(store.dat0)) build(); } };
  }

  // ------------------------------------------------------------------ view: diagnostics

  function highlightJson(value) {
    const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
    const pre = h('pre', { class: 'code' });
    const re = /("(?:\\.|[^"\\])*")(\s*:)?|\b(true|false|null)\b|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g;
    let last = 0, match;
    while ((match = re.exec(text))) {
      if (match.index > last) pre.append(text.slice(last, match.index));
      if (match[1]) {
        pre.append(h('span', { class: match[2] ? 'k' : 's', text: match[1] }));
        if (match[2]) pre.append(match[2]);
      } else if (match[3]) pre.append(h('span', { class: 'b', text: match[3] }));
      else pre.append(h('span', { class: 'n', text: match[4] }));
      last = re.lastIndex;
    }
    pre.append(text.slice(last));
    return pre;
  }

  function mountDiagnostics(main) {
    let requests = [];
    const snapshotButton = h('button', { class: 'btn btn-sm', type: 'button', onclick: snapshot }, icon('download'), t('snapshot'));
    main.append(h('div', { class: 'spread' }, h('div', null, h('h1', { class: 'section-title', text: t('diag_title') }), h('p', { class: 'section-sub', text: t('diag_sub') })), snapshotButton));

    const linkCard = card(t('link'), 'activity');
    const linkPill = h('span', { class: 'pill' });
    linkCard.head.append(linkPill);
    const linkBody = h('div');
    linkCard.el.append(linkBody);
    const processCard = card(t('process'), 'chip');
    const processBody = h('div');
    processCard.el.append(processBody);
    const countersCard = card(t('counters'), 'list');
    const countersBody = h('div');
    countersCard.el.append(countersBody);
    const requestsCard = card(t('requests'), 'send');
    const requestsBody = h('div', { class: 'table-wrap' });
    requestsCard.el.append(requestsBody);
    const freshnessCard = card(t('freshness'), 'clock');
    const freshnessBody = h('div');
    freshnessCard.el.append(freshnessBody);

    const rawCard = card(t('raw_data'), 'terminal');
    const rawPages = [['api/status', () => store.status], ['api/inf', () => store.inf], ['api/dat/0', () => store.dat0],
      ['api/dat/1', () => store.dat1], ['api/dat/2', () => store.dat2], ['api/features', () => store.features]];
    const rawBlocks = rawPages.map(([path, read]) => {
      const holder = h('div', { style: { padding: '0 8px 8px' } });
      const copyButton = h('button', { class: 'btn btn-sm btn-ghost', type: 'button', text: t('copy'),
        onclick: (event) => { event.preventDefault(); copyText(JSON.stringify(read(), null, 2), copyButton); } });
      const details = h('details', { class: 'raw' }, h('summary', null, h('span', { class: 'mono', text: 'GET /' + path }), copyButton), holder);
      details.addEventListener('toggle', () => { if (details.open) fill(); });
      const fill = () => { holder.textContent = ''; holder.append(highlightJson(read() ?? null)); };
      return { details, fill };
    });
    rawCard.el.append(...rawBlocks.map((b) => b.details));

    main.append(h('div', { class: 'stack' },
      h('div', { class: 'grid grid-2' }, linkCard.el, processCard.el), countersCard.el, requestsCard.el,
      h('div', { class: 'grid grid-2' }, freshnessCard.el, rawCard.el)));

    const kv = (rows) => h('dl', { class: 'kv' }, rows.map(([k, v]) => [h('dt', { text: k }), h('dd', null, v ?? '—')]));

    async function loadRequests() {
      const r = await api('GET', 'api/requests');
      if (r.ok) requests = r.data;
    }

    function statusPill(status) {
      const tone = { ok: 'pill-good', error: 'pill-crit', timeout: 'pill-warn', sent: 'pill-info', pending: 'pill-muted' }[status] || 'pill-muted';
      return h('span', { class: `pill ${tone}`, text: status });
    }

    async function snapshot() {
      await loadRequests();
      downloadJson(`hottoh-snapshot-${new Date().toISOString().replace(/[:.]/g, '-')}.json`, {
        generated_at: new Date().toISOString(), page: location.href, user_agent: navigator.userAgent,
        status: store.status, inf: store.inf, dat0: store.dat0, dat1: store.dat1, dat2: store.dat2, features: store.features, requests
      });
    }

    let busy = false;
    async function update() {
      const st = store.status;
      if (st) {
        linkPill.className = `pill ${st.connected ? 'pill-good pill-live' : 'pill-crit'}`;
        linkPill.textContent = '';
        linkPill.append(h('span', { class: 'dot' }), t(st.connected ? 'connected' : 'disconnected'));
        linkBody.textContent = '';
        // An error older than the current connection is over
        const resolved = st.last_error && st.connected_since && parseDate(st.last_error_at) <= parseDate(st.connected_since);
        linkBody.append(kv([
          [t('stove_address'), h('span', null, h('span', { class: 'mono', text: st.stove_address || '—' }),
            st.discovery ? h('span', { class: 'muted small', text: ` · ${t('disc_found')}` }) : null)],
          [t('connected_since'), st.connected_since ? `${fmtDateTime(parseDate(st.connected_since), true)} (${fmtDuration(ageSeconds(st.connected_since))})` : '—'],
          [t('last_response'), st.last_response_at ? `${fmtDateTime(parseDate(st.last_response_at), true)} (${fmtAgo(st.last_response_at)})` : t('never')],
          [t('connections'), fmtNum(st.connections)],
          [t('last_error'), st.last_error ? h('span', { class: resolved ? 'muted' : '' },
            h('span', { class: 'mono', text: st.last_error }), h('br'),
            h('span', { class: 'small', text: fmtDateTime(parseDate(st.last_error_at), true) + (resolved ? ` · ${t('error_resolved', { time: fmtDateTime(parseDate(st.connected_since)) })}` : '') })) : t('none')]
        ]));
        processBody.textContent = '';
        processBody.append(kv([
          [t('version'), st.version], [t('started'), fmtDateTime(parseDate(st.started_at), true)], [t('uptime'), fmtDuration(st.uptime_s)],
          [t('memory'), `${fmtNum(st.process.rss_kb / 1024, 1)} MiB`], [t('threads'), fmtNum(st.process.threads)],
          [t('fds'), fmtNum(st.process.open_fds)], [t('pending_writes'), fmtNum(st.pending_writes)]
        ]));
        const stats = st.stats;
        const bad = new Set(['timeouts', 'invalid_frames', 'decode_errors', 'writes_refused', 'writes_failed', 'reads_refused', 'reads_failed', 'disconnections']);
        countersBody.textContent = '';
        countersBody.append(h('p', { class: 'muted small', style: { marginBottom: '10px' }, text: t('counters_hint') }), h('div', { class: 'stats-grid' },
          h('div', { class: 'stat' }, h('b', { text: stats.answers ? `${fmtNum(stats.latency_total_ms / stats.answers)} ms` : '—' }), h('span', { text: t('latency_avg') })),
          h('div', { class: 'stat' }, h('b', { text: `${fmtNum(stats.latency_max_ms)} ms` }), h('span', { text: t('latency_max') })),
          Object.entries(stats).filter(([k]) => !k.startsWith('latency')).map(([k, v]) =>
            h('div', { class: 'stat' + (bad.has(k) && v > 0 ? ' bad' : '') }, h('b', { text: fmtNum(v) }), h('span', { text: I18N[lang]['c_' + k] ? t('c_' + k) : k })))));
      }

      freshnessBody.textContent = '';
      freshnessBody.append(kv([['INF', store.inf], ['DAT 0', store.dat0], ['DAT 1', store.dat1], ['DAT 2', store.dat2]].map(([name, page]) =>
        [name, received(page) ? `${fmtDateTime(parseDate(page.last_updated), true)} (${fmtAgo(page.last_updated)})` : t('never')])));
      rawBlocks.forEach((b) => { if (b.details.open) b.fill(); });

      if (busy) return;
      busy = true;
      await loadRequests();
      busy = false;
      requestsBody.textContent = '';
      if (!requests.length) { requestsBody.append(h('div', { class: 'empty', text: t('requests_empty') })); return; }
      requestsBody.append(h('table', null,
        h('thead', null, h('tr', null, [t('id'), t('created'), t('command'), t('value'), t('status'), t('attempts'), ''].map((x) => h('th', { text: x })))),
        h('tbody', null, requests.map((r) => h('tr', null,
          h('td', { class: 'mono', text: r.request_id }), h('td', { text: fmtDateTime(parseDate(r.created_at), true) }),
          h('td', { text: r.command }), h('td', { class: 'mono wrap', text: r.value }), h('td', null, statusPill(r.status)),
          h('td', { text: r.attempts }),
          h('td', { class: 'muted small wrap', text: r.error_code !== undefined ? `${r.error_code} · ${r.message || errorText(r.error_code)}` : r.message || '' }))))));
    }

    return { update };
  }

  // ------------------------------------------------------------------ view: console

  function mountConsole(main) {
    main.append(h('h1', { class: 'section-title', text: t('console_title') }), h('p', { class: 'section-sub', text: t('console_sub') }));
    let spec = null;
    const endpointSelect = h('select', { 'aria-label': t('endpoint'), style: { gridColumn: '1 / -1' } }, h('option', { value: '', text: t('endpoint') }));
    const methodSelect = h('select', { 'aria-label': 'Method' }, ['GET', 'POST'].map((m) => h('option', { value: m, text: m })));
    const pathInput = h('input', { type: 'text', value: '/api/status', spellcheck: 'false', autocomplete: 'off', 'aria-label': 'Path', class: 'mono' });
    const sendButton = h('button', { class: 'btn btn-primary send', type: 'button', onclick: send }, icon('send'), t('send'));
    const bodyInput = h('textarea', { rows: 6, spellcheck: 'false', 'aria-label': t('body'), placeholder: '{ }' });
    const bodyBlock = h('div', { hidden: true }, h('div', { class: 'field-label', text: t('body') }), bodyInput,
      h('div', { class: 'note warn', style: { marginTop: '8px' } }, icon('alert', { width: 16, height: 16 }), t('console_warn')));
    const responseMeta = h('div', { class: 'row' });
    const responseBody = h('div', null, h('div', { class: 'empty', text: t('no_response') }));
    const followLog = h('div', { class: 'stack', style: { gap: '4px' } });

    pathInput.addEventListener('keydown', (event) => { if (event.key === 'Enter') send(); });
    methodSelect.addEventListener('change', () => { bodyBlock.hidden = methodSelect.value !== 'POST'; });

    const requestCard = card(t('console_title'), 'terminal',
      h('div', { class: 'stack' }, h('div', { class: 'console-form' }, endpointSelect, methodSelect, pathInput, sendButton), bodyBlock));
    requestCard.head.append(h('a', { class: 'btn btn-sm', href: 'swagger-ui/', target: '_blank', rel: 'noopener' }, icon('external'), 'Swagger UI'));
    const responseCard = card(t('response'), 'send', h('div', { class: 'stack' }, responseMeta, followLog, responseBody));
    const activityList = h('div');
    const activityCard = card(t('activity'), 'list', activityList);
    activityCard.head.append(h('button', { class: 'btn btn-sm btn-ghost', type: 'button', text: t('clear'),
      onclick: () => { activity = []; storage.set('activity', activity); renderActivity(); } }));
    main.append(h('div', { class: 'stack' }, requestCard.el, h('div', { class: 'grid grid-2', style: { alignItems: 'start' } }, responseCard.el, activityCard.el)));

    function exampleBody(operation) {
      let schema = operation?.requestBody?.content?.['application/json']?.schema;
      const resolve = (sc) => (sc && sc.$ref ? spec.components?.schemas?.[sc.$ref.split('/').pop()] : sc);
      schema = resolve(schema);
      if (!schema) return '';
      const build = (sc) => {
        sc = resolve(sc);
        if (!sc) return null;
        if (sc.example !== undefined) return sc.example;
        if (sc.properties) return Object.fromEntries(Object.entries(sc.properties).map(([k, v]) => [k, build(v)]));
        const type = Array.isArray(sc.type) ? sc.type.find((x) => x !== 'null') : sc.type;
        return { boolean: false, integer: 0, number: 0, string: '', array: [] }[type] ?? null;
      };
      return JSON.stringify(build(schema), null, 2);
    }

    api('GET', 'api-docs/openapi.json').then((r) => {
      if (!r.ok || !r.data?.paths) return;
      spec = r.data;
      const groups = {};
      for (const [path, ops] of Object.entries(spec.paths)) {
        for (const [method, op] of Object.entries(ops)) {
          const tag = op.tags?.[0] || 'api';
          (groups[tag] ||= []).push({ path, method: method.toUpperCase(), op });
        }
      }
      for (const [tag, list] of Object.entries(groups)) {
        endpointSelect.append(h('optgroup', { label: tag }, list.map((e) => h('option', {
          value: `${e.method} ${e.path}`, text: `${e.method} ${e.path}`, title: e.op.summary || ''
        }))));
      }
      endpointSelect.addEventListener('change', () => {
        if (!endpointSelect.value) return;
        const [method, path] = endpointSelect.value.split(' ');
        methodSelect.value = method;
        pathInput.value = path;
        bodyBlock.hidden = method !== 'POST';
        if (method === 'POST') bodyInput.value = exampleBody(spec.paths[path].post);
        pathInput.focus();
      });
    });

    async function send() {
      const method = methodSelect.value;
      const path = pathInput.value.trim().replace(/^\/+/, '');
      let body;
      if (method === 'POST') {
        const text = bodyInput.value.trim() || '{}';
        try { JSON.parse(text); } catch (error) { showToast(t('invalid_json', { error: error.message }), 'error'); return; }
        body = text;
      }
      sendButton.disabled = true;
      followLog.textContent = '';
      const r = await api(method, path, body);
      sendButton.disabled = false;
      logActivity({ method, path: '/' + path, body: body !== undefined ? JSON.parse(body) : undefined, status: r.status, ms: r.ms, source: 'console' });
      responseMeta.textContent = '';
      const tone = r.status === 0 ? 'pill-crit' : r.ok ? 'pill-good' : r.status >= 500 ? 'pill-crit' : 'pill-warn';
      responseMeta.append(h('span', { class: `pill ${tone}`, text: r.status ? String(r.status) : 'network' }),
        h('span', { class: 'muted small num', text: `${fmtNum(r.ms)} ms · ${fmtNum(r.text.length)} B` }),
        h('span', { class: 'mono small', text: `${method} /${path}` }));
      const copyButton = h('button', { class: 'btn btn-sm btn-ghost', type: 'button', text: t('copy'), onclick: () => copyText(r.text, copyButton) });
      responseMeta.append(copyButton);
      responseBody.textContent = '';
      responseBody.append(highlightJson(r.data !== null && typeof r.data === 'object' ? r.data : r.text || r.error || ''));
      if (r.ok && r.data && typeof r.data === 'object' && r.data.request_id !== undefined) {
        const id = r.data.request_id;
        const line = (text) => followLog.append(h('div', { class: 'note', text }));
        line(t('following', { id }));
        const outcome = await followRequest(id, (st) => line(t('followed', { id, status: st.status })));
        if (outcome.status === 'error') line(errorText(outcome.error_code));
        refresh();
      }
    }

    function renderActivity() {
      activityList.textContent = '';
      if (!activity.length) { activityList.append(h('div', { class: 'empty', text: t('activity_empty') })); return; }
      for (const entry of activity) {
        activityList.append(h('button', { class: 'history-item', type: 'button', title: entry.body ? JSON.stringify(entry.body) : entry.path,
          onclick: () => {
            methodSelect.value = entry.method;
            pathInput.value = entry.path;
            bodyBlock.hidden = entry.method !== 'POST';
            bodyInput.value = entry.body !== undefined ? JSON.stringify(entry.body, null, 2) : '';
            window.scrollTo({ top: 0, behavior: 'smooth' });
          } },
        h('span', { class: 'muted num', text: new Date(entry.at).toLocaleTimeString(lang, { hour: '2-digit', minute: '2-digit' }) }),
        h('span', { class: `method ${entry.method.toLowerCase()}`, text: entry.method }),
        h('span', { class: 'path', text: entry.path }),
        h('span', { class: `pill ${entry.status >= 200 && entry.status < 300 ? 'pill-good' : 'pill-crit'}`, text: entry.status || '—' })));
      }
    }

    renderActivity();
    activityListeners.add(renderActivity);
    return { destroy: () => activityListeners.delete(renderActivity) };
  }

  // ------------------------------------------------------------------ shell: tabs, banner, router

  const VIEWS = {
    dashboard: { icon: 'flame', mount: mountDashboard },
    schedule: { icon: 'calendar', mount: mountSchedule },
    history: { icon: 'chart', mount: mountHistory },
    module: { icon: 'wifi', mount: mountModule },
    diagnostics: { icon: 'activity', mount: mountDiagnostics },
    console: { icon: 'terminal', mount: mountConsole }
  };
  let current = null;

  function renderTabs(active) {
    const nav = $('#tabs');
    nav.textContent = '';
    for (const [name, view] of Object.entries(VIEWS)) {
      nav.append(h('a', { class: 'tab', href: `#/${name}`, 'aria-current': name === active ? 'page' : false, title: t('nav_' + name) },
        icon(view.icon), h('span', { text: t('nav_' + name) })));
    }
  }

  async function route() {
    const name = (location.hash.match(/^#\/(\w+)/) || [])[1];
    const view = VIEWS[name] ? name : 'dashboard';
    if (current && current.name === view) return;
    if (current?.instance.dirty?.() && !(await confirmDialog({ title: t('discard_title'), text: t('discard_text') }))) {
      history.replaceState(null, '', `#/${current.name}`);
      return;
    }
    current?.instance.destroy?.();
    const main = $('#view');
    main.textContent = '';
    current = { name: view, instance: VIEWS[view].mount(main) || {} };
    renderTabs(view);
    document.title = `${t('nav_' + view)} · HottoH`;
    current.instance.update?.();
    updateChrome();
    window.scrollTo(0, 0);
  }

  function remount() {
    const name = current?.name;
    current?.instance.destroy?.();
    current = null;
    if (name) history.replaceState(null, '', `#/${name}`);
    route();
  }

  function updateChrome() {
    const st = store.status, d0 = store.dat0;
    const pill = $('#link-pill'), text = $('#link-text');
    const banner = $('#banner');
    let tone = 'pill-good pill-live', label = t('link_ok'), bannerText = null, bannerTone = 'crit';
    // Status read in the last polling cycles: an old one (tab in the background) says nothing
    const fresh = st && Date.now() - statusAt < 3 * POLL_MS;
    if (store.apiError) {
      tone = 'pill-crit'; label = t('link_api'); bannerText = t('banner_api', { error: store.apiError });
    } else if (st && !st.stove_address) {
      tone = 'pill-info pill-live'; label = t('link_searching');
    } else if (fresh && !st.connected) {
      tone = 'pill-crit'; label = t('link_down'); bannerText = t('banner_down', { error: st.last_error || '—' });
    } else if (fresh && ageSeconds(st.last_response_at) > 20) {
      tone = 'pill-warn'; label = t('link_stale'); bannerText = t('banner_stale', { age: fmtDuration(ageSeconds(st.last_response_at)) }); bannerTone = 'warn';
    } else if (received(d0)) {
      const state = stateInfo(d0);
      // The dashboard shows the alarm in its own card
      if (state.alarm && current?.name !== 'dashboard') { bannerText = t('banner_alarm', { state: state.label }); bannerTone = state.tone === 'warn' ? 'warn' : 'crit'; }
    }
    pill.className = `pill ${tone}`;
    text.textContent = label;
    pill.title = label;
    banner.hidden = !bannerText;
    if (bannerText) {
      banner.className = `banner ${bannerTone}`;
      banner.textContent = '';
      banner.append(icon('alert'), h('span', { text: bannerText }));
    }
    const inf = store.inf;
    $('#brand-sub').textContent = received(inf) ? `${inf.hostname} · ${inf.version}` : st?.stove_address || '—';
    $('#footer-version').textContent = st ? `hottoh_api ${st.version}` : 'hottoh_api';
  }

  const THEMES = ['auto', 'light', 'dark'];
  let theme = storage.get('theme', 'auto');

  function applyTheme() {
    if (theme === 'auto') document.documentElement.removeAttribute('data-theme');
    else document.documentElement.setAttribute('data-theme', theme);
    const button = $('#theme-btn');
    button.textContent = '';
    button.append(icon(theme === 'light' ? 'sun' : theme === 'dark' ? 'moon' : 'auto'));
    button.title = t('theme_' + theme);
    button.setAttribute('aria-label', t('theme_' + theme));
  }

  function applyLang() {
    document.documentElement.lang = lang;
    const button = $('#lang-btn');
    button.textContent = lang === 'fr' ? 'EN' : 'FR';
    button.title = lang === 'fr' ? 'English' : 'Français';
    applyTheme();
  }

  $('#theme-btn').addEventListener('click', () => {
    theme = THEMES[(THEMES.indexOf(theme) + 1) % THEMES.length];
    storage.set('theme', theme);
    applyTheme();
    if (current?.name === 'history') remount();
  });
  $('#lang-btn').addEventListener('click', () => {
    lang = lang === 'fr' ? 'en' : 'fr';
    storage.set('lang', lang);
    applyLang();
    remount();
    updateChrome();
  });
  matchMedia('(prefers-color-scheme: dark)').addEventListener?.('change', () => { if (theme === 'auto' && current?.name === 'history') remount(); });

  window.addEventListener('hashchange', route);
  applyLang();
  route();
  startPolling();
})();
