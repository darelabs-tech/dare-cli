<?php

namespace App\Http\Controllers\Api;

use App\Http\Requests\LoginRequest;
use App\Services\AuthService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Routing\Controller;

class AuthController extends Controller
{
    public function __construct(private readonly AuthService $auth)
    {
    }

    public function login(LoginRequest $request): JsonResponse
    {
        $payload = $this->auth->login($request->validated());
        return response()->json($payload);
    }

    public function me(Request $request): JsonResponse
    {
        $user = $request->user();
        if ($user === null) {
            return response()->json(['error' => 'Unauthenticated'], 401);
        }
        return response()->json([
            'id' => $user->id,
            'email' => $user->email,
            'role' => $user->role,
            'createdAt' => $user->created_at?->toIso8601String(),
        ]);
    }
}
