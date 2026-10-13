<?php

use App\Http\Controllers\Api\AuthController;
use App\Http\Controllers\Api\UsersController;
use Illuminate\Support\Facades\Route;

Route::post('/auth/login', [AuthController::class, 'login']);

Route::middleware('auth:sanctum')->group(function () {
    Route::get('/auth/me', [AuthController::class, 'me']);
    Route::get('/users', [UsersController::class, 'index']);
    Route::post('/users', [UsersController::class, 'store']);
});
