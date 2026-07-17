# HelixEdu smoke — courses, publish/unpublish, soft-delete, enrollments, progress, withdraw
# Prereq: helix_edu_api on 8106, Postgres migrated, HELIX_ALLOW_DEV_HEADERS=1

$ErrorActionPreference = "Stop"
$h = @{ "x-helix-dev-user" = "ops@helixforge.local"; "Content-Type" = "application/json" }
$base = "http://127.0.0.1:8106"

function InvokeApi($Method, $Uri, $Body = $null) {
    if ($Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 15
    }
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $h -TimeoutSec 15
}

Write-Host "=== healthz ==="
$code = (Invoke-WebRequest "$base/healthz" -UseBasicParsing -TimeoutSec 8).StatusCode
if ($code -ne 200) { throw "healthz failed" }

Write-Host "=== domain status ==="
$st = (InvokeApi GET "$base/v1/domain/status").data
if ($st.phase -ne "wave2_w4") { throw "expected phase wave2_w4" }
if (-not $st.planes.soft_delete) { throw "expected soft_delete plane" }
if (-not $st.planes.withdraw) { throw "expected withdraw plane" }
if (-not $st.planes.progress_history) { throw "expected progress_history plane" }
if (-not $st.planes.publish_unpublish) { throw "expected publish_unpublish plane" }

Write-Host "=== create course ==="
$slug = "smoke-course-$(Get-Random)"
$course = (InvokeApi POST "$base/v1/courses" @{
    slug = $slug
    title = "Smoke Course"
    description = "HelixEdu smoke course"
    level = "intermediate"
}).data
if ($course.status -ne "draft") { throw "expected draft course" }

Write-Host "=== update course ==="
$updated = (InvokeApi PATCH "$base/v1/courses/$($course.id)" @{
    title = "Smoke Course Updated"
    description = "Updated description"
}).data
if ($updated.title -ne "Smoke Course Updated") { throw "course title not updated" }
if ($updated.description -ne "Updated description") { throw "course description not updated" }

Write-Host "=== publish course ==="
$published = (InvokeApi POST "$base/v1/courses/$($course.id)/publish" $null).data
if ($published.status -ne "published") { throw "expected published course" }

Write-Host "=== enroll learner ==="
$enrollment = (InvokeApi POST "$base/v1/enrollments" @{
    course_id = $published.id
    learner_label = "smoke-learner"
}).data
if ($enrollment.status -ne "active") { throw "expected active enrollment" }
if ($enrollment.progress_pct -ne 0) { throw "expected 0 progress" }

Write-Host "=== duplicate enrollment rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/enrollments" -Headers $h -Body (@{
        course_id = $published.id
        learner_label = "smoke-learner"
    } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 409 for duplicate enrollment"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 409) { throw "expected 409, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  409 as expected"
}

Write-Host "=== update progress to 50 ==="
$half = (InvokeApi POST "$base/v1/enrollments/$($enrollment.id)/progress" @{
    progress_pct = 50
}).data
if ($half.progress_pct -ne 50) { throw "expected progress 50" }
if ($half.status -ne "active") { throw "expected active at 50%" }

Write-Host "=== complete course ==="
$completed = (InvokeApi POST "$base/v1/enrollments/$($enrollment.id)/progress" @{
    progress_pct = 100
}).data
if ($completed.status -ne "completed") { throw "expected completed status" }
if (-not $completed.completed_at) { throw "expected completed_at timestamp" }

Write-Host "=== get enrollment ==="
$got = (InvokeApi GET "$base/v1/enrollments/$($enrollment.id)" $null).data
if ($got.id -ne $enrollment.id) { throw "get enrollment mismatch" }

Write-Host "=== withdraw enrollment ==="
$withdrawn = (InvokeApi POST "$base/v1/enrollments/$($enrollment.id)/withdraw" $null).data
if ($withdrawn.status -ne "withdrawn") { throw "expected withdrawn status" }

Write-Host "=== progress update on withdrawn enrollment rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/enrollments/$($enrollment.id)/progress" -Headers $h -Body (@{
        progress_pct = 75
    } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for progress on withdrawn enrollment"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== unpublish course ==="
$drafted = (InvokeApi POST "$base/v1/courses/$($course.id)/unpublish" $null).data
if ($drafted.status -ne "draft") { throw "expected draft after unpublish" }

Write-Host "=== enrollment into draft course rejected ==="
try {
    Invoke-RestMethod -Method POST -Uri "$base/v1/enrollments" -Headers $h -Body (@{
        course_id = $drafted.id
        learner_label = "smoke-learner-2"
    } | ConvertTo-Json) -TimeoutSec 8 | Out-Null
    throw "expected 422 for enrollment into draft course"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 422) { throw "expected 422, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  422 as expected"
}

Write-Host "=== soft-delete course ==="
$deleted = (InvokeApi POST "$base/v1/courses/$($course.id)/delete" $null).data
if ($deleted.id -ne $course.id) { throw "deleted course mismatch" }

Write-Host "=== get deleted course returns 404 ==="
try {
    Invoke-RestMethod -Method GET -Uri "$base/v1/courses/$($course.id)" -Headers $h -TimeoutSec 8 | Out-Null
    throw "expected 404 for deleted course"
}
catch {
    if ($_.Exception.Response.StatusCode -ne 404) { throw "expected 404, got $($_.Exception.Response.StatusCode)" }
    Write-Host "  404 as expected"
}

Write-Host "=== restore course ==="
$restored = (InvokeApi POST "$base/v1/courses/$($course.id)/restore" $null).data
if ($restored.id -ne $course.id) { throw "restored course mismatch" }
if ($restored.status -ne "draft") { throw "expected draft after restore" }

Write-Host "=== list courses ==="
$courses = (InvokeApi GET "$base/v1/courses" $null).data
$found = $courses.items | Where-Object { $_.id -eq $course.id }
if (-not $found) { throw "restored course not listed" }

Write-Host "=== product info ==="
$info = (InvokeApi GET "$base/v1/product" $null).data
if ($info.slug -ne "helix-edu") { throw "product slug mismatch" }

Write-Host ""
Write-Host "HELIX_EDU_SMOKE PASS"
exit 0
