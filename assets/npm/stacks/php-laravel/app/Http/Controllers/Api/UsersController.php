<?php

namespace App\Http\Controllers\Api;

use App\Http\Requests\CreateUserRequest;
use App\Services\UsersService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Routing\Controller;

class UsersController extends Controller
{
    public function __construct(private readonly UsersService $users)
    {
    }

    public function index(Request $request): JsonResponse
    {
        $page = max(1, (int) $request->query('page', '1'));
        $limit = min(100, max(1, (int) $request->query('limit', '20')));
        return response()->json($this->users->list($page, $limit));
    }

    public function store(CreateUserRequest $request): JsonResponse
    {
        $caller = $request->user();
        if ($caller === null || $caller->role !== 'ADMIN') {
            return response()->json(['error' => 'admin role required'], 403);
        }
        $created = $this->users->create($request->validated());
        return response()->json($created, 201);
    }
}
