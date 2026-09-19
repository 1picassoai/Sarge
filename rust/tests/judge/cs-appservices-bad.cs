using Microsoft.EntityFrameworkCore;
using WorkshopTools;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddDbContext<ToolContext>(o => o.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")));
var app = builder.Build();

app.MapGet("/tools", () =>
{
    var context = app.Services.GetRequiredService<ToolContext>();
    return context.Tools.ToList();
});

app.MapPost("/tools", async (Tool tool) =>
{
    using var context = app.Services.GetRequiredService<ToolContext>();
    context.Tools.Add(tool);
    await context.SaveChangesAsync();
    return Results.Created("", tool);
});
app.Run();
