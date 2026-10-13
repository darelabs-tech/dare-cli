<?php

return [
    'default' => env('REVERB_SERVER', 'reverb'),
    'servers' => [
        'reverb' => [
            'host' => env('REVERB_SERVER_HOST', '0.0.0.0'),
            'port' => (int) env('REVERB_SERVER_PORT', 8080),
            'hostname' => env('REVERB_HOST'),
            'options' => [
                'tls' => [],
            ],
            'max_request_size' => 10_000,
            'scaling' => [
                'enabled' => false,
                'channel' => 'reverb',
                'server' => [],
            ],
        ],
    ],
    'apps' => [
        'provider' => 'config',
        'apps' => [
            [
                'app_id' => (string) env('REVERB_APP_ID', 'myapp'),
                'key' => env('REVERB_APP_KEY'),
                'secret' => env('REVERB_APP_SECRET'),
                'ping_interval' => 60,
                'activity_timeout' => 30,
                'max_message_size' => 10_000,
            ],
        ],
    ],
];
