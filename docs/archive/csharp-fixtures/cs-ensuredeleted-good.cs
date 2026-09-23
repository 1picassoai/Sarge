using Microsoft.EntityFrameworkCore;
using WorkshopTools;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddDbContext<ToolContext>(o => o.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")));
var app = builder.Build();

using var scope = app.Services.CreateScope();
var context = scope.ServiceProvider.GetRequiredService<ToolContext>();
context.Database.EnsureCreated();

app.MapGet("/tools", async (ToolContext db) => await db.Tools.Where(t => !t.IsArchived).ToListAsync());
app.MapDelete("/tools/{id}", async (int id, ToolContext db) =>
{
    var tool = await db.Tools.FindAsync(id);
    if (tool is null) return Results.NotFound();
    tool.IsArchived = true;
    await db.SaveChangesAsync();
    return Results.NoContent();
});
app.Run();
