<?php

use Laravel\Sanctum\Sanctum;

return [
    'stateful' => explode(',', (string) env('SANCTUM_STATEFUL_DOMAINS', 'localhost,127.0.0.1')),
    'guard' => ['web'],
    'expiration' => (int) env('SANCTUM_TOKEN_EXPIRES_MIN', 60 * 24),
    'token_prefix' => env('SANCTUM_TOKEN_PREFIX', ''),
    'middleware' => [
        'authenticate_session' => Laravel\Sanctum\Http\Middleware\AuthenticateSession::class,
        'encrypt_cookies' => Illuminate\Cookie\Middleware\EncryptCookies::class,
        'validate_csrf_token' => Illuminate\Foundation\Http\Middleware\ValidateCsrfToken::class,
    ],
];
