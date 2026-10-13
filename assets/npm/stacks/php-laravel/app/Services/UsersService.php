<?php

namespace App\Services;

use App\Repositories\UsersRepository;
use Illuminate\Validation\ValidationException;

class UsersService
{
    public function __construct(private readonly UsersRepository $repo)
    {
    }

    /**
     * @return array{items: list<array<string, mixed>>, total: int, page: int}
     */
    public function list(int $page, int $limit): array
    {
        [$items, $total] = $this->repo->page($page, $limit);
        return [
            'items' => array_map([$this, 'toDto'], $items),
            'total' => $total,
            'page' => $page,
        ];
    }

    /**
     * @param  array<string, mixed>  $input
     * @return array<string, mixed>
     */
    public function create(array $input): array
    {
        $email = strtolower(trim((string) $input['email']));
        if ($this->repo->findByEmail($email) !== null) {
            throw ValidationException::withMessages(['email' => 'email already in use']);
        }
        $user = $this->repo->create([
            'email' => $email,
            'password' => bcrypt((string) $input['password']),
            'role' => $input['role'] ?? 'USER',
        ]);
        return $this->toDto($user);
    }

    /**
     * @return array<string, mixed>
     */
    private function toDto(\App\Models\User $u): array
    {
        return [
            'id' => $u->id,
            'email' => $u->email,
            'role' => $u->role,
            'createdAt' => $u->created_at?->toIso8601String(),
        ];
    }
}
