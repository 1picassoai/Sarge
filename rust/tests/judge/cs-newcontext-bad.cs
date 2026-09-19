using Microsoft.EntityFrameworkCore;
using WorkshopTools;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddDbContext<ToolContext>(o => o.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")));
var app = builder.Build();

app.MapGet("/tools", () =>
{
    using var context = new ToolContext(new DbContextOptionsBuilder<ToolContext>().UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")).Options);
    return context.Tools.ToList();
});

app.MapPost("/tools", async (Tool tool, ToolContext db) =>
{
    db.Tools.Add(tool);
    await db.SaveChangesAsync();
    return Results.Created($"/tools/{tool.Id}", tool);
});
app.Run();
