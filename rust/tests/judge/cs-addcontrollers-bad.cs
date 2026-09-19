using Microsoft.EntityFrameworkCore;
using WorkshopTools;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddControllers();

builder.Services.AddDbContext<ToolContext>(o => o.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")));
var app = builder.Build();

app.MapGet("/tools", async (ToolContext db) => await db.Tools.ToListAsync());
app.MapPost("/tools", async (Tool tool, ToolContext db) =>
{
    db.Tools.Add(tool);
    await db.SaveChangesAsync();
    return Results.Created($"/tools/{tool.Id}", tool);
});
app.Run();
