<?php

use App\Models\User;
use Laravel\Sanctum\Sanctum;

it('lists users when authenticated', function () {
    Sanctum::actingAs(User::query()->create([
        'email' => 'admin@example.test',
        'password' => bcrypt('Str0ngPass'),
        'role' => 'ADMIN',
    ]));

    $res = $this->getJson('/api/users');
    $res->assertOk();
    $res->assertJsonStructure(['items', 'total', 'page']);
});

it('rejects non-admin from creating users', function () {
    Sanctum::actingAs(User::query()->create([
        'email' => 'user@example.test',
        'password' => bcrypt('Str0ngPass'),
        'role' => 'USER',
    ]));

    $res = $this->postJson('/api/users', [
        'email' => 'new@example.test',
        'password' => 'Str0ngPass1',
    ]);
    $res->assertStatus(403);
});
