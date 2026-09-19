using Microsoft.EntityFrameworkCore;
using WorkshopTools;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddDbContext<ToolContext>(o => o.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")));
var app = builder.Build();

using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<ToolContext>();
    db.Database.EnsureCreated();
}

app.MapGet("/tools", async (ToolContext db) => await db.Tools.Where(t => !t.IsArchived).Select(t => new { t.Id, t.Name, t.Price }).ToListAsync());
app.MapGet("/tools/{id}", async (int id, ToolContext db) =>
{
    var tool = await db.Tools.FindAsync(id);
    return tool is null || tool.IsArchived ? Results.NotFound() : Results.Ok(tool);
});
app.MapPost("/tools", async (Tool tool, ToolContext db) =>
{
    db.Tools.Add(tool);
    await db.SaveChangesAsync();
    return Results.Created($"/tools/{tool.Id}", tool);
});
app.Run();

namespace WorkshopTools
{
    public class Tool { public int Id { get; set; } public string Name { get; set; } = ""; public decimal Price { get; set; } public bool IsArchived { get; set; } }
    public class ToolContext : DbContext { public DbSet<Tool> Tools { get; set; } public ToolContext(DbContextOptions<ToolContext> o) : base(o) { } }
}
